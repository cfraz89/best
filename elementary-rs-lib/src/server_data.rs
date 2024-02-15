use std::collections::HashMap;

use bevy_ecs::entity::Entity;

use crate::{node::Node, selector::Selector, world::WORLD};

#[cfg_attr(target_arch = "wasm32", derive(serde::Deserialize, Debug))]
#[cfg_attr(not(target_arch = "wasm32"), derive(serde::Serialize, Debug))]
#[derive(bevy_ecs::component::Component)]
pub struct ServerData(HashMap<String, serde_json::Value>);

impl ServerData {
    pub fn get_serial_server_data(entity: &Entity) -> SerialServerData {
        let world = WORLD.read().unwrap();
        let entity_ref = world.entity(*entity);
        let node = entity_ref.get::<Node>().expect("Entity needs a node");
        let selector = entity_ref
            .get::<Selector>()
            .expect("need selector on entity");
        let server_data = entity_ref
            .get::<ServerData>()
            .expect("Need server data on entity");
        let mut server_data_map = HashMap::new();
        server_data_map.insert(selector.to_string(), server_data.0);
        push_node_server_data(&mut SerialServerData(server_data_map), node);
        SerialServerData(server_data_map)
    }
}

/// Walk the tree and push server data
fn push_node_server_data(into: &mut SerialServerData, node: &Node) {
    match node {
        Node::Component {
            entity,
            child_nodes,
        } => {
            into.0
                .extend(ServerData::get_serial_server_data(entity).0.into_iter());
            for child in child_nodes.iter() {
                push_node_server_data(into, child);
            }
        }
        Node::HtmlElement {
            element: _,
            child_nodes,
        } => {
            for child in child_nodes.iter() {
                push_node_server_data(into, child)
            }
        }
        _ => {}
    }
}

#[cfg_attr(target_arch = "wasm32", derive(serde::Deserialize, Debug))]
#[cfg_attr(not(target_arch = "wasm32"), derive(serde::Serialize, Debug))]
pub struct SerialServerData(pub HashMap<String, HashMap<String, serde_json::Value>>);

impl SerialServerData {
    pub fn get(&self, selector: &Selector) -> Option<ServerData> {
        self.0.get(&selector.to_string()).cloned().map(ServerData)
    }
}
