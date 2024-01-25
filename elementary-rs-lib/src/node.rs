use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use std::{collections::HashMap, fmt::Write, sync::Arc};

pub enum Node {
    Text(String),
    HtmlElement(HtmlElement),
    Component(Box<dyn Component>),
}

pub struct HtmlElement {
    pub tag: String,
    pub attributes: HashMap<String, String>,
    pub child_nodes: Arc<Vec<Node>>,
}

pub trait Component {
    fn node(&self) -> Node;
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
            Node::HtmlElement(HtmlElement {
                tag,
                attributes,
                child_nodes,
            }) => {
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
            Node::Component(component) => str
                .write_str(&component.node().render())
                .expect("couldn't write children"),
        }
        res
    }
}
