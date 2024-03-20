use std::rc::Rc;

use bevy::{ecs::system::EntityCommands, prelude::*};
use futures::task::Spawn;

pub struct IfNode<F> {
    pub condition: F,
    pub child_nodes: Vec<AnyChimeraNode>,
}
pub struct EntityNode<B> {
    pub bundle: B,
    pub child_nodes: Vec<AnyChimeraNode>,
}

pub enum AnyChimeraNode {
    Entity(Box<dyn AnyEntityNode>),
    If(Box<dyn AnyIfNode>),
}

macro_rules! define_node_trait {
    ($trait_name:ident, $return_type:ty) => {
        pub trait $trait_name {
            fn spawn(&self, commands: &mut Commands) -> $return_type;
            fn spawn_with_child_builder(&self, child_builder: &mut ChildBuilder) -> $return_type;
            fn spawn_with_world(&self, world: &mut World) -> $return_type;
            fn spawn_with_world_child_builder(
                &self,
                child_builder: &mut WorldChildBuilder,
            ) -> $return_type;
        }
    };
}

define_node_trait!(AnyEntityNode, Entity);
define_node_trait!(AnyIfNode, Option<Vec<Entity>>);

macro_rules! impl_entity_node_spawn {
    ($name:ident, $builder:ident, $builder_type: ident, $child_builder_func: ident) => {
        fn $name(&self, $builder: &mut $builder_type) -> Entity {
            let mut entity = $builder.spawn(self.bundle.clone());
            if self.child_nodes.len() > 0 {
                entity.with_children(|builder| {
                    for child in &self.child_nodes {
                        match child {
                            AnyChimeraNode::Entity(c) => {
                                c.$child_builder_func(builder);
                            }
                            AnyChimeraNode::If(c) => {
                                c.$child_builder_func(builder);
                            }
                        }
                    }
                });
            }
            entity.id()
        }
    };
}

impl<T: Bundle + Clone> AnyEntityNode for EntityNode<T> {
    impl_entity_node_spawn!(spawn, commands, Commands, spawn_with_child_builder);
    impl_entity_node_spawn!(
        spawn_with_child_builder,
        child_builder,
        ChildBuilder,
        spawn_with_child_builder
    );
    impl_entity_node_spawn!(
        spawn_with_world,
        world,
        World,
        spawn_with_world_child_builder
    );
    impl_entity_node_spawn!(
        spawn_with_world_child_builder,
        world,
        WorldChildBuilder,
        spawn_with_world_child_builder
    );
}

macro_rules! impl_if_node_spawn {
    ($name:ident, $builder:ident, $builder_type: ident) => {
        fn $name(&self, $builder: &mut $builder_type) -> Option<Vec<Entity>> {
            if (self.condition)() {
                Some(
                    self.child_nodes
                        .iter()
                        .flat_map(|c| match c {
                            AnyChimeraNode::Entity(c) => vec![c.$name($builder)],
                            AnyChimeraNode::If(c) => c.$name($builder).unwrap_or_default(),
                        })
                        .collect(),
                )
            } else {
                None
            }
        }
    };
}

impl<F: Fn() -> bool> AnyIfNode for IfNode<F> {
    impl_if_node_spawn!(spawn, commands, Commands);
    impl_if_node_spawn!(spawn_with_child_builder, child_builder, ChildBuilder);
    impl_if_node_spawn!(spawn_with_world, world, World);
    impl_if_node_spawn!(
        spawn_with_world_child_builder,
        child_builder,
        WorldChildBuilder
    );
}
