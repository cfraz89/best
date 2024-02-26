use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
};

use bevy_ecs::entity::Entity;

use crate::{
    node::{Node, NodeRef},
    selector::Selector,
    world::WORLD,
};

#[cfg_attr(target_arch = "wasm32", derive(serde::Deserialize, Debug))]
#[cfg_attr(not(target_arch = "wasm32"), derive(serde::Serialize, Debug))]
#[derive(bevy_ecs::component::Component)]
pub struct ServerData(HashMap<String, serde_json::Value>);

impl Default for ServerData {
    fn default() -> Self {
        ServerData(HashMap::new())
    }
}

impl Deref for ServerData {
    type Target = HashMap<String, serde_json::Value>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ServerData {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl ServerData {
    pub fn get_serial_server_data(entity: &Entity) -> SerialServerData {
        let world = WORLD.read().unwrap();
        let entity_ref = world.entity(*entity);
        let node_ref = entity_ref.get::<NodeRef>().expect("Entity needs a node");
        let selector = entity_ref
            .get::<Selector>()
            .expect("need selector on entity");
        let server_data = entity_ref.get::<ServerData>();
        let mut server_data_map = HashMap::new();
        if let Some(server_data) = server_data {
            server_data_map.insert(selector.to_string(), server_data.0.clone());
        }
        let mut serial_map = SerialServerData(server_data_map);
        serial_map.extract_node_server_data(node_ref);
        serial_map
    }
}

#[cfg_attr(target_arch = "wasm32", derive(serde::Deserialize, Debug))]
#[cfg_attr(
    not(target_arch = "wasm32"),
    derive(serde::Serialize, serde::Deserialize, Debug)
)]
#[derive(Clone)]
pub struct SerialServerData(pub HashMap<String, HashMap<String, serde_json::Value>>);

impl Deref for SerialServerData {
    type Target = HashMap<String, HashMap<String, serde_json::Value>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for SerialServerData {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl SerialServerData {
    pub fn get(&self, selector: &Selector) -> Option<ServerData> {
        self.0.get(&selector.to_string()).cloned().map(ServerData)
    }

    /// Walk the tree and extract server data from all the node's child component entities into our serial_server_data
    fn extract_node_server_data(&mut self, from: &NodeRef) {
        match from.as_ref() {
            Node::Component(
                entity,
                // child_nodes,
            ) => {
                self.extend(ServerData::get_serial_server_data(entity).0.into_iter());
                // for child in child_nodes.iter() {
                //     self.extract_node_server_data(child);
                // }
            }
            Node::HtmlElement {
                element: _,
                child_nodes,
            } => {
                for child in child_nodes.iter() {
                    self.extract_node_server_data(child)
                }
            }
            _ => {}
        }
    }
}
