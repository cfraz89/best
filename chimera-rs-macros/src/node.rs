use std::sync::Arc;

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

#[derive(Debug, Clone)]
pub enum ChimeraMacroNode {
    Entity {
        components: Vec<TokenStream>,
        child_nodes: Arc<Vec<ChimeraMacroNode>>,
    },
    If {
        condition: TokenStream,
        child_nodes: Arc<Vec<ChimeraMacroNode>>,
    },
}

impl ChimeraMacroNode {
    fn to_tokens_any_node(&self) -> proc_macro2::TokenStream {
        match self {
            ChimeraMacroNode::If { .. } => {
                quote! {
                    chimera_rs::node::AnyChimeraNode::If(Box::new(#self))
                }
            }
            ChimeraMacroNode::Entity { .. } => {
                quote! {
                    chimera_rs::node::AnyChimeraNode::Entity(Box::new(#self))
                }
            }
        }
    }
}

/// Macrotic writing out TemplateNode -> Node
impl ToTokens for ChimeraMacroNode {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        {
            match self {
                ChimeraMacroNode::If {
                    condition,
                    child_nodes,
                } => {
                    let child_nodes = child_nodes.iter().map(|c| c.to_tokens_any_node());
                    tokens.extend(quote! {
                        chimera_rs::node::IfNode {
                            condition: move || #condition,
                            child_nodes: vec![#(#child_nodes),*],
                        }
                    })
                }
                ChimeraMacroNode::Entity {
                    components,
                    child_nodes,
                } => {
                    let child_nodes = child_nodes.iter().map(|c| c.to_tokens_any_node());
                    tokens.extend(quote! {
                        chimera_rs::node::EntityNode {
                            bundle: (#(#components),*),
                            child_nodes: vec![#(#child_nodes),*],
                        }
                    })
                }
            }
        }
    }
}
