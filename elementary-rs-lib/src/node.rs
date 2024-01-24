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

pub trait Component: ToTokens {
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

impl ToTokens for Node {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        {
            tokens.extend(match self {
                Node::HtmlElement(HtmlElement {
                    tag,
                    attributes,
                    child_nodes,
                }) => {
                    let stream = TokenStream::new();
                    quote! {
                    elementary_rs_lib::node::Node::HtmlElement(elementary_rs_lib::node::HtmlElement {
                        tag: #tag.to_string(),
                        attributes: Default::default(),
                        child_nodes: Arc::new(vec![#(#child_nodes),*]),
                    })
                }
                }
                Node::Text(text) => quote! {
                    elementary_rs_lib::node::Node::Text(#text.to_string())
                }
                .into(),
                Node::Component(component) => quote! {
                    elementary_rs_node::node::Node::Component(Box::new(#component))
                }
                .into(),
            })
        }
    }
}
