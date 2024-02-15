use std::fmt::Display;

use bevy_ecs::component::Component;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Component)]
pub enum Selector {
    Id(String),
    Class(String),
}

impl Default for Selector {
    fn default() -> Self {
        Selector::Class("component".to_string())
    }
}

impl Display for Selector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Selector::Id(id) => write!(f, "#_{}", id),
            Selector::Class(class) => write!(f, "._{}", class),
        }
    }
}
