use bevy::prelude::*;

#[derive(Component, Debug, Clone)]
pub struct Text(pub &'static str);
