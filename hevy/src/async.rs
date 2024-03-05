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
use std::sync::mpsc::{Receiver, Sender};
use std::task::{Context, Poll::Ready};

#[derive(Resource)]
pub struct AsyncTasks {
    pub(crate) map:
        HashMap<Entity, HashMap<usize, Pin<Box<dyn Future<Output = ()> + Sync + Send + 'static>>>>,
    pub(crate) world_callback_tx: Sender<Box<dyn Fn(&mut World) -> () + Send + Sync>>,
    pub(crate) commands_callback_tx: Sender<Box<dyn Fn(Commands) -> () + Send + Sync>>,
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
    commands_tx: Sender<Box<dyn Fn(Commands) -> () + Send + Sync + 'static>>,
}

impl AsyncCallbacks {
    pub fn with_world(&self, cb: impl Fn(&mut World) -> () + Send + Sync + 'static) {
        self.world_tx.send(Box::new(cb)).unwrap();
    }
    pub fn with_commands(&self, cb: impl Fn(Commands) -> () + Send + Sync + 'static) {
        self.commands_tx.send(Box::new(cb)).unwrap();
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

pub fn update_tasks(mut async_tasks: ResMut<AsyncTasks>, mut context: NonSendMut<AsyncContext>) {
    match context.0.get_mut() {
        Option::None => {}
        Option::Some(context) => {
            let mut completed_entities = Vec::<Entity>::new();
            for (entity, hm) in async_tasks.map.iter_mut() {
                let mut completed_ids = Vec::<usize>::new();
                for (id, action) in hm.iter_mut() {
                    if let Ready(_) = action.poll(context.deref_mut()) {
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
    };
}

pub(crate) struct AsyncReceivers {
    pub(crate) world_callback_rx: Receiver<Box<dyn Fn(&mut World) -> () + Send + Sync + 'static>>,
    pub(crate) commands_callback_rx: Receiver<Box<dyn Fn(Commands) -> () + Send + Sync + 'static>>,
}

pub(crate) fn process_async_callbacks(world: &mut World) {
    let receivers = world.non_send_resource::<Rc<AsyncReceivers>>().clone();
    while let Ok(cb) = receivers.world_callback_rx.try_recv() {
        cb(world);
    }
    let mut command_queue = CommandQueue::default();
    while let Ok(cb) = receivers.commands_callback_rx.try_recv() {
        let commands = Commands::new(&mut command_queue, world);
        cb(commands);
    }
    command_queue.apply(world);
    // let mut context = world.non_send_resource_mut::<AsyncContext>();
    // match context.deref_mut() {
    //     AsyncContext::Dropped => {}
    //     AsyncContext::Context(context) => {
    //         context.get_mut().waker().wake_by_ref();
    //     }
    // }
}
