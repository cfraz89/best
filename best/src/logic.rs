use bevy::prelude::*;

#[derive(Component, Debug, Clone)]
pub struct If(pub bool);

#[derive(Component, Debug, Clone)]
pub struct Else;
