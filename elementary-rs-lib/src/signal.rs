use std::{
    marker::PhantomData,
    ops::{Deref},
    sync::{Arc, Mutex},
};

use bevy_ecs::{component::Component, entity::Entity};

use crate::world::WORLD;

#[derive(Clone)]
pub struct Signal<T> {
    entity: Entity,
    data: PhantomData<T>, // val: Arc<Mutex<T>>,
                          // observers:
                          //     Arc<Mutex<Vec<Box<dyn Fn(&T) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> + Send>>>>,
}

#[derive(Component)]
pub struct SignalValue<T>(Arc<Mutex<T>>);

impl<T> Deref for SignalValue<T> {
    type Target = Arc<Mutex<T>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> SignalValue<T> {
    fn new(value: T) -> Self {
        SignalValue(Arc::new(Mutex::new(value)))
    }
}

impl<T: Default + Send + Sync + 'static> Default for Signal<T> {
    fn default() -> Self {
        let mut world = WORLD.write().unwrap();
        let entity_ref = world.spawn(SignalValue::<T>(Default::default()));
        Self {
            entity: entity_ref.id(),
            data: PhantomData::default(),
        }
    }
}

impl<T: Send + Sync + 'static> Signal<T> {
    pub fn new(val: T) -> Self {
        let mut world = WORLD.write().unwrap();
        let entity_ref = world.spawn(SignalValue::new(val));
        Self {
            entity: entity_ref.id(),
            data: PhantomData::default(),
        }
    }

    // pub fn set(&self, val: T) {
    //     *self.val.lock().unwrap() = val;
    // }
}

impl<T: Send + Sync + 'static + Clone> Signal<T> {
    pub fn get(&self) -> T {
        let world = WORLD.read().unwrap();
        let entity_ref = world.entity(self.entity);
        let value = entity_ref.get::<SignalValue<T>>().unwrap();
        let ret = value.lock().unwrap().clone();
        ret
    }

    // pub fn subscribe(
    //     &self,
    //     on_change: Box<dyn Fn(&T) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> + Send>,
    // ) {
    //     self.observers.lock().unwrap().push(on_change);
    // }
}
