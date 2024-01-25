use std::{collections::HashMap, sync::Arc};

use proc_macro2::{Ident, TokenStream, TokenTree};
use quote::{format_ident, quote, ToTokens};

#[derive(Debug)]
pub enum TemplateNode {
    Text(String),
    HtmlElement(HtmlElement),
    ComponentElement(ComponentElement),
}

#[derive(Debug)]
pub struct ComponentElement {
    pub name: String,
    pub properties: HashMap<Ident, TokenStream>,
}

#[derive(Debug)]
pub struct HtmlElement {
    pub tag: String,
    pub attributes: HashMap<String, String>,
    pub child_nodes: Arc<Vec<TemplateNode>>,
}

impl ToTokens for TemplateNode {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        {
            tokens.extend(match self {
                TemplateNode::HtmlElement(HtmlElement {
                    tag,
                    attributes,
                    child_nodes,
                }) => {
                    quote! {
                    elementary_rs_lib::node::Node::HtmlElement(elementary_rs_lib::node::HtmlElement {
                        tag: #tag.to_string(),
                        attributes: Default::default(),
                        child_nodes: Arc::new(vec![#(#child_nodes),*]),
                    })
                }
                }
                TemplateNode::Text(text) => quote! {
                    elementary_rs_lib::node::Node::Text(#text.to_string())
                }
                .into(),
                TemplateNode::ComponentElement(ComponentElement{name,properties}) => {
                  let name_ident = format_ident!("{}", name);
                  let properties = properties.iter().map(|(k,v)| {
                    quote! {
                      #k: #v
                    }
                  });
                  quote! {
                    elementary_rs_lib::node::Node::Component(Box::new(#name_ident {
                        #(#properties),*
                    }
                    ))}.into()}})
        }
    }
}
