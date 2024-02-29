use std::collections::HashMap;

use bevy::prelude::*;

use super::tag::Tag;

/// Attributes that can actually be set when you're feeling lazy
#[derive(Component, Debug)]
pub struct Attributes(pub HashMap<String, String>);

///Attributes that are rendered when the page is built
#[derive(Component, Debug)]
pub struct RenderAttributes(pub HashMap<String, String>);

/// Every update stage render attributes are reset to nothing
pub fn reset_render_attributes(mut commands: Commands, query: Query<Entity, With<Tag>>) {
    for entity in &query {
        commands
            .entity(entity)
            .insert(RenderAttributes(HashMap::new()));
    }
}

/// Build render attributes out of attributes
pub fn add_attributes_to_render_attributes(
    mut commands: Commands,
    query: Query<(Entity, &Attributes, &RenderAttributes)>,
) {
    for (entity, attributes, render_attributes) in &query {
        let mut new_attributes = render_attributes.0.clone();
        //This should overwrite old attributes when there are conflicts
        new_attributes.extend(attributes.0.clone());
        commands
            .entity(entity)
            .insert(RenderAttributes(new_attributes));
    }
}
