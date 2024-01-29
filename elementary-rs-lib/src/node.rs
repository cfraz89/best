use std::{collections::HashMap, fmt::Write, sync::Arc};

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

pub trait Renderable {
    fn render(&self) -> String;
}

impl Renderable for Node {
    fn render(&self) -> String {
        let mut res = String::new();
        let str = &mut res;
        match self {
            Node::Text(string) => write!(str, "{}", string).expect("couldn't write text"),
            Node::HtmlElement {
                element: HtmlElement { tag, attributes },
                child_nodes,
            } => {
                let attributes_string = attributes
                    .into_iter()
                    .map(|(k, v)| format!(" {}=\"{}\"", k, v))
                    .collect::<Vec<String>>()
                    .join(" ");
                write!(str, "<{}{}>", tag, attributes_string).expect("couldn't write opening tag");
                for child in child_nodes.iter() {
                    str.write_str(&child.render()).expect("couldn't child");
                }
                write!(str, "</{}>", tag).expect("couldn't write closing tag");
            }
            Node::Component {
                element,
                child_nodes,
            } => write!(
                str,
                "<{}><template shadowrootmode=\"open\">{}</template>{}</{}>",
                element.tag(),
                element.node().render(),
                child_nodes
                    .iter()
                    .map(|child| child.render())
                    .collect::<Vec<String>>()
                    .join("\n"),
                element.tag()
            )
            .expect("couldn't write component"),
        }
        res
    }
}
