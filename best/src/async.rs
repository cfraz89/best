use bevy::ecs::system::CommandQueue;
use bevy::prelude::*;
use bevy::tasks::futures_lite::FutureExt;
use std::cell::RefCell;
use std::collections::HashMap;
use std::future::Future;
use std::mem::transmute;
use std::ops::{Deref, DerefMut};
use std::pin::Pin;
use std::task::Waker;
use std::task::{Context, Poll::Ready};
use tokio::sync::mpsc::{Receiver, Sender};

#[derive(Resource)]
pub struct AsyncTasks {
    pub(crate) map:
        HashMap<Entity, HashMap<usize, Pin<Box<dyn Future<Output = ()> + Sync + Send + 'static>>>>,
    pub(crate) world_callback_tx: Sender<Box<dyn Fn(&mut World) -> () + Send + Sync>>,
}

impl AsyncTasks {
    pub fn run_async<F: Future<Output = ()> + Send + Sync + 'static>(
        &mut self,
        entity: Entity,
        future: impl FnOnce(WorldCallback) -> F,
    ) {
        let world_cb = WorldCallback {
            world_tx: self.world_callback_tx.clone(),
        };
        let hm = self.map.entry(entity).or_insert_with(HashMap::new);
        hm.insert(hm.len(), Box::pin(future(world_cb)));
    }
}

pub struct WorldCallback {
    world_tx: Sender<Box<dyn Fn(&mut World) -> () + Send + Sync + 'static>>,
}

impl WorldCallback {
    pub async fn with_world(&self, cb: impl Fn(&mut World) -> () + Send + Sync + 'static) {
        self.world_tx.send(Box::new(cb)).await.unwrap();
    }
}

/// Poll all async tasks stored against entities, and remove them if they are polled to completion
pub(crate) fn update_tasks(mut async_tasks: ResMut<AsyncTasks>, waker: Res<AsyncWaker>) {
    match &waker.0 {
        None => {}
        Some(waker) => {
            let mut context = Context::from_waker(&waker);
            let mut completed_entities = Vec::<Entity>::new();

            for (entity, hm) in async_tasks.map.iter_mut() {
                let mut completed_ids = Vec::<usize>::new();
                for (id, action) in hm.iter_mut() {
                    if let Ready(_) = action.poll(&mut context) {
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
    }
}

#[derive(Resource)]
pub(crate) struct AsyncRx {
    pub(crate) world_callback_rx: Receiver<Box<dyn Fn(&mut World) -> () + Send + Sync + 'static>>,
}

/// Run async with_world and with_commands callbacks
pub(crate) fn process_world_callbacks(world: &mut World) {
    while let Ok(cb) = world.resource_mut::<AsyncRx>().world_callback_rx.try_recv() {
        cb(world);
    }
}

#[derive(Resource)]
pub struct AsyncWaker(pub Option<Waker>);
