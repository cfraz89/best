use std::{
    collections::HashMap,
    fmt::{self, Display, Write},
    sync::Arc,
};

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
    Expression(Box<dyn Fn() -> String>),
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
                    write!(f, " />");
                } else {
                    write!(f, ">")?;
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
            Node::Expression(exp_fn) => write!(f, "{}", exp_fn()),
        }
    }
}
