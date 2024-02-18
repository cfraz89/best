use bevy_ecs::component::Component;

#[derive(Component, Clone)]
pub struct HydrationFnName(pub String);
