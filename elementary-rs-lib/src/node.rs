use std::{
    collections::HashMap,
    fmt::{self, Display, Write},
    sync::Arc,
};

use web_sys::{console, wasm_bindgen::JsValue, window};

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
            // document.evaluate("//h1[contains(., 'Hello')]", document, null, XPathResult.ANY_TYPE, null );
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
            Node::HtmlElement {
                element,
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
            _ => Ok(()),
        }
    }
}
