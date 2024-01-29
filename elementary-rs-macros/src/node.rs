use std::{collections::HashMap, sync::Arc};

use proc_macro2::{Ident, TokenStream, TokenTree};
use quote::{format_ident, quote, ToTokens};

#[derive(Debug)]
pub enum TemplateNode {
    Text(String),
    HtmlElement {
        element: HtmlElement,
        child_nodes: Arc<Vec<TemplateNode>>,
    },
    ComponentElement {
        element: ComponentElement,
        child_nodes: Arc<Vec<TemplateNode>>,
    },
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
}

impl ToTokens for TemplateNode {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        {
            tokens.extend(match self {
                TemplateNode::HtmlElement {
                    element: HtmlElement { tag, attributes },
                    child_nodes,
                } => {
                    quote! {
                        elementary_rs_lib::node::Node::HtmlElement {
                            element: elementary_rs_lib::node::HtmlElement {
                                tag: #tag.to_string(),
                                attributes: Default::default(),
                            },
                            child_nodes: Arc::new(vec![#(#child_nodes),*])
                        }
                    }
                }
                TemplateNode::Text(text) => quote! {
                    elementary_rs_lib::node::Node::Text(#text.to_string())
                }
                .into(),
                TemplateNode::ComponentElement {
                    element: ComponentElement { name, properties },
                    child_nodes,
                } => {
                    let name_ident = format_ident!("{}", name);
                    let properties = properties.iter().map(|(k, v)| {
                        quote! {
                        #k: #v
                        }
                    });
                    quote! {
                        elementary_rs_lib::node::Node::Component {
                            element: Box::new(
                                #name_ident {
                                    #(#properties),*
                                }
                            ),
                            child_nodes: Arc::new(vec![#(#child_nodes),*])
                        }
                    }
                    .into()
                }
            })
        }
    }
}
