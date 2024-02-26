use crate::node::NodeRef;
use bevy_ecs::prelude::*;
use std::future::Future;
use std::ops::{Deref, DerefMut};
use std::pin::Pin;
use std::sync::Arc;

#[derive(Component, Debug, Clone)]
pub struct HydrationFnName(pub String);

#[derive(Component, Debug, Clone)]
pub struct JSPath(pub String);

#[derive(Component, Debug, Clone)]
pub struct Tag(pub String);

pub trait WebComponent {
    fn template(self) -> impl Future<Output = NodeRef> + Send + '_;
}

pub trait DynWebComponent {
    fn template(self) -> Pin<Box<dyn Future<Output = NodeRef> + Send + '_>>;
}

impl<T: WebComponent<template(): Send> + 'static> DynWebComponent for T {
    fn template(self) -> Pin<Box<dyn Future<Output = NodeRef> + Send + '_>> {
        Box::pin(WebComponent::template(self))
    }
}

#[derive(Component, Clone)]
pub struct AnyWebComponent(pub Arc<dyn DynWebComponent + Sync + Send>);

impl<T: WebComponent<template(): Send> + Sync + Send + 'static> From<T> for AnyWebComponent {
    fn from(component: T) -> Self {
        AnyWebComponent(Arc::new(component))
    }
}

impl Deref for AnyWebComponent {
    type Target = Arc<dyn DynWebComponent + Sync + Send>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for AnyWebComponent {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Component)]
pub struct WebComponentChildren(pub Vec<NodeRef>);

///Used by derive macro to construct our entities
pub trait BuildWebComponent {
    fn build_entity(self, commands: Commands, child_nodes: Vec<NodeRef>) -> Entity;
}
