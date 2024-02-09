use std::{
    collections::HashMap,
    sync::{Arc, Mutex, OnceLock},
};

use crate::node::Node;

pub trait ComponentContext {
    fn context(&self) -> &Context;
}

#[cfg_attr(target_arch = "wasm32", derive(serde::Deserialize, Debug))]
#[cfg_attr(not(target_arch = "wasm32"), derive(serde::Serialize, Debug))]
pub struct Context {
    pub server_data: Mutex<HashMap<String, serde_json::Value>>,
    #[serde(skip)]
    pub view: Arc<OnceLock<Node>>,
}

impl Default for Context {
    fn default() -> Self {
        Self {
            server_data: Mutex::new(HashMap::new()),
            view: Arc::new(OnceLock::new()),
        }
    }
}
