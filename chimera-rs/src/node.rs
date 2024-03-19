use bevy::prelude::*;

pub struct IfNode<F> {
    pub condition: F,
    pub child_nodes: Vec<Box<dyn ChimeraNode>>,
}
pub struct EntityNode<B> {
    pub bundle: B,
    pub child_nodes: Vec<Box<dyn ChimeraNode>>,
}

pub trait ChimeraNode {
    fn spawn(&self, commands: &mut Commands) -> Vec<Entity>;
    fn world_spawn(&self, world: &mut World) -> Vec<Entity>;
    fn child_spawn(&self, builder: &mut ChildBuilder) -> Vec<Entity>;
    fn world_child_spawn(&self, world: &mut WorldChildBuilder) -> Vec<Entity>;
}

impl<T: Bundle + Clone> ChimeraNode for EntityNode<T> {
    fn spawn(&self, commands: &mut Commands) -> Vec<Entity> {
        let mut entity = commands.spawn(self.bundle.clone());
        if self.child_nodes.len() > 0 {
            entity.with_children(|builder| {
                for child in &self.child_nodes {
                    child.child_spawn(builder);
                }
            });
        }
        vec![entity.id()]
    }

    fn world_spawn(&self, world: &mut World) -> Vec<Entity> {
        let mut entity = world.spawn(self.bundle.clone());
        if self.child_nodes.len() > 0 {
            entity.with_children(|builder| {
                for child in &self.child_nodes {
                    child.world_child_spawn(builder);
                }
            });
        }
        vec![entity.id()]
    }

    fn world_child_spawn(&self, builder: &mut WorldChildBuilder) -> Vec<Entity> {
        let mut entity = builder.spawn(self.bundle.clone());
        if self.child_nodes.len() > 0 {
            entity.with_children(|builder| {
                for child in &self.child_nodes {
                    child.world_child_spawn(builder);
                }
            });
        }
        vec![entity.id()]
    }

    fn child_spawn(&self, builder: &mut ChildBuilder) -> Vec<Entity> {
        let mut entity = builder.spawn(self.bundle.clone());
        if self.child_nodes.len() > 0 {
            entity.with_children(|builder| {
                for child in &self.child_nodes {
                    child.child_spawn(builder);
                }
            });
        }
        vec![entity.id()]
    }
}

impl<F: Fn() -> bool> ChimeraNode for IfNode<F> {
    fn spawn(&self, commands: &mut Commands) -> Vec<Entity> {
        if (self.condition)() {
            self.child_nodes
                .iter()
                .flat_map(|c| c.spawn(commands))
                .collect()
        } else {
            Vec::new()
        }
    }

    fn world_spawn(&self, world: &mut World) -> Vec<Entity> {
        if (self.condition)() {
            self.child_nodes
                .iter()
                .flat_map(|c| c.world_spawn(world))
                .collect()
        } else {
            Vec::new()
        }
    }

    fn world_child_spawn(&self, builder: &mut WorldChildBuilder) -> Vec<Entity> {
        if (self.condition)() {
            self.child_nodes
                .iter()
                .flat_map(|c| c.world_child_spawn(builder))
                .collect()
        } else {
            Vec::new()
        }
    }

    fn child_spawn(&self, builder: &mut ChildBuilder) -> Vec<Entity> {
        if (self.condition)() {
            self.child_nodes
                .iter()
                .flat_map(|c| c.child_spawn(builder))
                .collect()
        } else {
            Vec::new()
        }
    }
}
