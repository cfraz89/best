use std::sync::{LazyLock, RwLock};

use bevy_ecs::world::World;

pub static WORLD: LazyLock<RwLock<World>> = LazyLock::new(|| RwLock::new(World::default()));
