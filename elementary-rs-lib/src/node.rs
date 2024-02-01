use std::{
    collections::HashMap,
    fmt::{self, Display, Write},
    sync::Arc,
};

use web_sys::{wasm_bindgen::JsValue, window};

pub enum Node {
    Text(String),
    HtmlElement {
        element: HtmlElement,
        child_nodes: Arc<Vec<Node>>,
    },
    Component {
        element: Box<dyn Component>,
        child_nodes: Arc<Vec<Node>>,
    },
    Expression(String, Box<dyn Fn() -> String>),
}

// impl From<&'static Node> for ClientNode {
//     fn from(node: &'static Node) -> Self {
//         match node {
//             Node::Text(string) => ClientNode {
//                 child_nodes: vec![],
//                 expression: None,
//             },
//             Node::HtmlElement {
//                 element,
//                 child_nodes,
//             } => ClientNode {
//                 child_nodes: child_nodes
//                     .iter()
//                     .map(|node| node.into())
//                     .collect::<Vec<ClientNode>>(),
//                 expression: None,
//             },
//             Node::Component {
//                 element,
//                 child_nodes,
//             } => ClientNode {
//                 child_nodes: child_nodes.iter().map(|node| node.into()).collect(),
//                 expression: None,
//             },
//             Node::Expression(id, exp_fn) => ClientNode {
//                 child_nodes: vec![],
//                 expression: Some(ClientExpression {
//                     id: id.to_string(),
//                     expr: Closure::wrap(Box::new(exp_fn.to_owned())),
//                 }),
//             },
//         }
//     }
// }

// #[wasm_bindgen]
// pub struct ClientNode {
//     child_nodes: Vec<ClientNode>,
//     expression: Option<ClientExpression>,
// }

// #[wasm_bindgen]
// struct ClientExpression {
//     id: String,
//     expr: Closure<dyn Fn() -> String>,
// }

// impl ClientNode {
//     pub fn bind(&self) -> Result<(), JsValue> {
//         for child in self.child_nodes.iter() {
//             child.bind()?;
//         }
//         if let Some(ClientExpression { ref id, ref expr }) = self.expression {
//             let document = window()
//                 .expect("No window")
//                 .document()
//                 .expect("no document");
//             let result = document
//                 .query_selector(format!("slot#{id}").as_str())?
//                 .unwrap();
//             result.set_text_content(Some(
//                 format!(
//                     "JS: {}",
//                     expr.as_ref()
//                         .unchecked_ref::<Function>()
//                         .call0(&JsValue::default())?
//                         .as_string()
//                         .unwrap()
//                 )
//                 .as_str(),
//             ));
//         }
//         Ok(())
//     }
// }

#[derive(Clone)]
pub struct HtmlElement {
    pub tag: String,
    pub attributes: HashMap<String, String>,
}

pub trait Component: CustomElement {
    fn node(&self) -> Node;
}

pub trait CustomElement {
    fn tag(&self) -> &'static str;
}

impl Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Node::Text(string) => write!(f, "{}", string),
            Node::HtmlElement {
                element: HtmlElement { tag, attributes },
                child_nodes,
            } => {
                let attributes_string = attributes
                    .into_iter()
                    .map(|(k, v)| format!(" {}=\"{}\"", k, v))
                    .collect::<Vec<String>>()
                    .join(" ");
                write!(f, "<{}{}", tag, attributes_string)?;
                if child_nodes.is_empty() {
                    write!(f, " />")?;
                } else {
                    write!(f, " >")?;
                    for child in child_nodes.iter() {
                        child.fmt(f)?;
                    }
                    write!(f, "</{}>", tag)?;
                }
                Ok(())
            }
            Node::Component {
                element,
                child_nodes,
            } => {
                let mut children_str = String::new();
                for child in child_nodes.iter() {
                    write!(children_str, "{}", child)?;
                }
                write!(
                    f,
                    "<{}><template shadowrootmode=\"open\">{}</template>{}</{}>",
                    element.tag(),
                    element.node(),
                    children_str,
                    element.tag()
                )
            }
            Node::Expression(id, exp_fn) => {
                write!(f, "<slot id=\"{}\">{}</slot>", id, exp_fn())
            }
        }
    }
}

impl Node {
    // pub fn web_node(&self) -> Result<web_sys::Node, JsValue> {
    //     match self {
    //         Node::Text(string) => Ok(web_sys::Text::new_with_data(string)?.into()),
    //         Node::HtmlElement {
    //             element: HtmlElement { tag, attributes },
    //             child_nodes,
    //         } => {
    //             let element = web_sys::window()
    //                 .expect("No window")
    //                 .document()
    //                 .expect("no document")
    //                 .create_element(tag)?;
    //             for child in child_nodes.iter() {}
    //             Ok(element.into())
    //         }
    //         Node::Component {
    //             element,
    //             child_nodes,
    //         } => {
    //             let element = web_sys::window()
    //                 .expect("No window")
    //                 .document()
    //                 .expect("no document")
    //                 .create_element(element.tag())?;
    //             Ok(element.into())
    //         }
    //         Node::Expression(uuid, exp_fn) => {
    //             let text = web_sys::Text::new_with_data(&exp_fn())?;
    //             //Todo implement signal listeners here
    //             Ok(text.into())
    //         }
    //     }
    // }

    pub fn bind(&self) -> Result<(), JsValue> {
        match self {
            Node::Text(_) => Ok(()),
            Node::HtmlElement {
                element: HtmlElement { tag, attributes },
                child_nodes,
            } => {
                for child in child_nodes.iter() {
                    child.bind()?;
                }
                Ok(())
            }
            Node::Component {
                element,
                child_nodes,
            } => {
                for child in child_nodes.iter() {
                    child.bind()?;
                }
                Ok(())
            }
            Node::Expression(id, expr) => {
                let document = window()
                    .expect("No window")
                    .document()
                    .expect("no document");
                let result = document
                    .query_selector(format!("slot#{id}").as_str())?
                    .unwrap();
                result.set_text_content(Some(format!("JS: {}", expr()).as_str()));
                Ok(())
            }
        }
    }
}
