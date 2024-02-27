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

pub trait WebComponent: Clone {
    fn template(self, world: &mut World) -> impl Future<Output = NodeRef> + Send;
}

pub trait DynWebComponent {
    fn template<'a>(
        &'a self,
        world: &'a mut World,
    ) -> Pin<Box<dyn Future<Output = NodeRef> + Send + '_>>;
}

impl<T: WebComponent<template(): Send> + 'static> DynWebComponent for T {
    fn template<'a>(
        &'a self,
        world: &'a mut World,
    ) -> Pin<Box<dyn Future<Output = NodeRef> + Send + '_>> {
        Box::pin(WebComponent::template(self.clone(), world))
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
    fn build_entity(self, world: &mut World, child_nodes: Vec<NodeRef>) -> Entity;
}

///Used to manage async building of templates
#[derive(Component)]
pub struct BuildTemplate(pub AnyWebComponent);

#[derive(Component)]
pub struct Template(pub NodeRef);
