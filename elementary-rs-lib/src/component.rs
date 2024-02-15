pub trait Component:
    bevy_ecs::component::Component + serde::Serialize + for<'a> serde::Deserialize<'a>
{
    fn build_entity(self) -> bevy_ecs::entity::Entity;
}
