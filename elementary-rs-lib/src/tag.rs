use std::fmt::Display;

use bevy_ecs::component::Component;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Component)]
pub struct Tag(pub String);
