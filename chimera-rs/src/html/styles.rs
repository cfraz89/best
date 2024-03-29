use std::collections::HashMap;

use bevy::prelude::*;

use super::attributes::RenderAttributes;

#[derive(Component, Debug, Clone)]
pub struct Styles(pub HashMap<&'static str, &'static str>);

pub fn apply_styles(mut commands: Commands, query: Query<(Entity, &RenderAttributes, &Styles)>) {
    for (entity, render_attributes, style) in &query {
        let mut new_attributes = render_attributes.0.clone();
        let added_style = style
            .0
            .iter()
            .map(|(k, v)| format!("{}:{};", k, v))
            .collect::<Vec<_>>()
            .join("");
        new_attributes
            .entry("style".to_string())
            .or_default()
            .push_str(&added_style);
        commands
            .entity(entity)
            .insert(RenderAttributes(new_attributes));
    }
}
