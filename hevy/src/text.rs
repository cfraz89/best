use bevy::prelude::*;

#[derive(Component, Debug, Clone)]
pub struct Text(pub &'static str);

#[derive(Component, Debug, Clone)]
pub struct TextString(pub String);
