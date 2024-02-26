use std::{fmt::Formatter, ops::Deref, sync::Arc};

use bevy_ecs::{component::Component, entity::Entity};

/// https://developer.mozilla.org/en-US/docs/Web/API/Document/createTreeWalker
/// Ashamedly had to reference leptos here
const SHOW_COMMENT: u32 = 0x80;

pub enum Node {
    Text(String),
    HtmlElement {
        element: HtmlElement,
        child_nodes: Vec<NodeRef>,
    },
    Component(Entity),
    Expression(String, Box<dyn Fn() -> String + Send + Sync>),
}

impl std::fmt::Debug for Node {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Node::Text(s) => write!(f, "Text({s})"),
            Node::HtmlElement {
                element,
                child_nodes,
            } => write!(f, "HtmlElement({:?}, {:?})", element, child_nodes),
            Node::Component(entity) => write!(f, "Component({:?})", entity),
            Node::Expression(e, _expr) => write!(f, "Expression({e})"),
        }
    }
}

#[derive(Component, Clone, Debug)]
pub struct NodeRef(Arc<Node>);

impl From<Node> for NodeRef {
    fn from(node: Node) -> NodeRef {
        NodeRef(Arc::new(node))
    }
}

impl Deref for NodeRef {
    type Target = Arc<Node>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug)]
pub struct HtmlElement {
    pub tag: String,
    pub attributes: Vec<(String, String)>,
}
