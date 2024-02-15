use std::{
    borrow::BorrowMut,
    future::Future,
    ops::{Deref, DerefMut},
    pin::Pin,
    sync::{Arc, Mutex, MutexGuard},
};

#[derive(Clone)]
pub struct Signal<T> {
    val: Arc<Mutex<T>>,
    observers:
        Arc<Mutex<Vec<Box<dyn Fn(&T) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> + Send>>>>,
}

impl<T> Signal<T> {
    pub fn new(val: T) -> Self {
        Signal {
            val: Arc::new(Mutex::new(val)),
            observers: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn set(&self, val: T) {
        *self.val.lock().unwrap() = val;
    }

    pub fn get(&self) -> MutexGuard<'_, T> {
        self.val.lock().unwrap()
    }

    pub fn subscribe(
        &self,
        on_change: Box<dyn Fn(&T) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> + Send>,
    ) {
        self.observers.lock().unwrap().push(on_change);
    }
}
