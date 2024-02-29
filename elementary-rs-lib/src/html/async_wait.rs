use bevy::prelude::*;

#[derive(Component, Debug, Clone)]
pub struct AsyncWait(pub usize);
