use bevy::{
    ecs::{bundle::Bundle, entity::Entity, system::Commands},
    hierarchy::BuildChildren,
};

pub fn set_child<'w, 's>(
    mut commands: Commands<'w, 's>,
    entity: Entity,
    child_bundle: impl Bundle,
) {
    commands.entity(entity).clear_children();
    commands.entity(entity).with_children(|builder| {
        builder.spawn(child_bundle);
    });
}
