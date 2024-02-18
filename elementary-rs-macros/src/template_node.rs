use std::{collections::HashMap, sync::Arc};

use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote, ToTokens};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

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
    Expression(TokenStream),
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

/// Macrotic writing out TemplateNode -> Node
impl ToTokens for TemplateNode {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        {
            tokens.extend(match self {
                TemplateNode::HtmlElement {
                    element: HtmlElement { tag, attributes: _ },
                    child_nodes,
                } => {
                    quote! {
                        elementary_rs_lib::node::NodeRef::from(elementary_rs_lib::node::Node::HtmlElement {
                            element: elementary_rs_lib::node::HtmlElement {
                                tag: #tag.to_string(),
                                attributes: Default::default(),
                            },
                            child_nodes: vec![#(#child_nodes),*]
                        })
                    }
                }
                TemplateNode::Text(text) => quote! {
                    elementary_rs_lib::node::NodeRef::from(elementary_rs_lib::node::Node::Text(#text.to_string()))
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
                        elementary_rs_lib::node::NodeRef::from(elementary_rs_lib::node::Node::Component {
                            entity: elementary_rs_lib::component::Component::build_entity(#name_ident {
                                        #(#properties),*
                                    }),
                            child_nodes: vec![#(#child_nodes),*]
                        })
                    }
                    .into()
                }
                TemplateNode::Expression(tokens) => {
                    let mut hasher = DefaultHasher::new();
                    tokens.to_string().hash(&mut hasher);
                    //Adding the e to make it a valid identifier
                    let hash = hasher.finish();
                    quote! {
                        elementary_rs_lib::node::NodeRef::from(elementary_rs_lib::node::Node::Expression(#hash.to_string(), Box::new({
                            move || (#tokens).to_string()
                    })))
                    }
                }
            })
        }
    }
}
