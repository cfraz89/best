use bevy::ecs::system::CommandQueue;
use bevy::prelude::*;
use bevy::tasks::futures_lite::FutureExt;
use std::cell::RefCell;
use std::collections::HashMap;
use std::future::Future;
use std::mem::transmute;
use std::ops::{Deref, DerefMut};
use std::pin::Pin;
use std::rc::Rc;
use std::sync::{Arc, RwLock};
use std::task::{Context, Poll::Ready};
use tokio::sync::mpsc::{Receiver, Sender};

#[derive(Resource)]
pub struct AsyncTasks {
    pub(crate) map:
        HashMap<Entity, HashMap<usize, Pin<Box<dyn Future<Output = ()> + Sync + Send + 'static>>>>,
    pub(crate) world_callback_tx: Sender<Box<dyn Fn(&mut World) -> () + Send + Sync>>,
    pub(crate) commands_callback_tx: Sender<Box<dyn Fn(&mut Commands) -> () + Send + Sync>>,
}

impl AsyncTasks {
    pub fn run<F: Future<Output = ()> + Send + Sync + 'static>(
        &mut self,
        entity: Entity,
        future: impl FnOnce(AsyncCallbacks) -> F,
    ) {
        let callbacks = AsyncCallbacks {
            world_tx: self.world_callback_tx.clone(),
            commands_tx: self.commands_callback_tx.clone(),
        };
        let hm = self.map.entry(entity).or_insert_with(HashMap::new);
        hm.insert(hm.len(), Box::pin(future(callbacks)));
    }
}

pub struct AsyncCallbacks {
    world_tx: Sender<Box<dyn Fn(&mut World) -> () + Send + Sync + 'static>>,
    commands_tx: Sender<Box<dyn Fn(&mut Commands) -> () + Send + Sync + 'static>>,
}

impl AsyncCallbacks {
    pub async fn with_world(&self, cb: impl Fn(&mut World) -> () + Send + Sync + 'static) {
        self.world_tx.send(Box::new(cb)).await.unwrap();
    }
    pub async fn with_commands(&self, cb: impl Fn(&mut Commands) -> () + Send + Sync + 'static) {
        self.commands_tx.send(Box::new(cb)).await.unwrap();
    }
}

/// Non-Send
pub struct AsyncContext(RefCell<Option<&'static mut Context<'static>>>);

impl Deref for AsyncContext {
    type Target = RefCell<Option<&'static mut Context<'static>>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for AsyncContext {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<&mut Context<'_>> for AsyncContext {
    fn from(context: &mut Context<'_>) -> Self {
        // Should be safe as long as AsyncContext is updated to dropped after the update tasj
        Self(RefCell::new(Option::Some(unsafe {
            transmute::<&mut Context, &mut Context>(context)
        })))
    }
}

impl AsyncContext {
    pub fn new() -> Self {
        Self(RefCell::new(Option::None))
    }
}

pub fn update_tasks(world: &mut World, context: &mut Context<'_>) {
    let mut async_tasks = world.resource_mut::<AsyncTasks>();
    let mut completed_entities = Vec::<Entity>::new();
    for (entity, hm) in async_tasks.map.iter_mut() {
        let mut completed_ids = Vec::<usize>::new();
        for (id, action) in hm.iter_mut() {
            if let Ready(_) = action.poll(context) {
                completed_entities.push(*entity);
                completed_ids.push(*id);
            }
        }
        for id in completed_ids {
            hm.remove(&id);
        }
    }
    for entity in completed_entities {
        match async_tasks.map.get(&entity) {
            Some(hm) if hm.len() == 0 => {
                async_tasks.map.remove(&entity);
            }
            _ => {}
        }
    }
}

#[derive(Resource)]
pub(crate) struct AsyncReceivers {
    pub(crate) world_callback_rx:
        Arc<RwLock<Receiver<Box<dyn Fn(&mut World) -> () + Send + Sync + 'static>>>>,
    pub(crate) commands_callback_rx:
        Arc<RwLock<Receiver<Box<dyn Fn(&mut Commands) -> () + Send + Sync + 'static>>>>,
}

pub(crate) fn process_async_callbacks(world: &mut World) {
    let world_callback_rx = {
        world
            .resource_mut::<AsyncReceivers>()
            .world_callback_rx
            .clone()
    };
    while let Ok(cb) = world_callback_rx.write().unwrap().try_recv() {
        cb(world);
    }

    let mut command_queue = CommandQueue::default();
    let commands_callback_rx = {
        world
            .resource_mut::<AsyncReceivers>()
            .commands_callback_rx
            .clone()
    };
    while let Ok(cb) = commands_callback_rx.write().unwrap().try_recv() {
        let mut commands = Commands::new(&mut command_queue, world);
        cb(&mut commands);
    }
    command_queue.apply(world);
}
