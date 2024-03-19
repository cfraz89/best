use std::sync::Arc;

use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};

#[derive(Debug, Clone)]
pub enum ChimeraMacroNode {
    Entity {
        bundle: TokenStream,
        child_nodes: Arc<Vec<ChimeraMacroNode>>,
    },
    If {
        condition: TokenStream,
        child_nodes: Arc<Vec<ChimeraMacroNode>>,
    },
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
                    let child_nodes = child_nodes.iter().map(|c| {
                        quote! {
                            Box::new(#c) as Box<dyn chimera_rs::node::ChimeraNode>
                        }
                    });
                    tokens.extend(quote! {
                        chimera_rs::node::IfNode {
                            condition: move || #condition,
                            child_nodes: vec![#(#child_nodes),*],
                        }
                    })
                }
                ChimeraMacroNode::Entity {
                    bundle,
                    child_nodes,
                } => {
                    let child_nodes = child_nodes.iter().map(|c| {
                        quote! {
                            Box::new(#c) as Box<dyn chimera_rs::node::ChimeraNode>
                        }
                    });
                    tokens.extend(quote! {
                        chimera_rs::node::EntityNode {
                            bundle: (#bundle),
                            child_nodes: vec![#(#child_nodes),*],
                        }
                    })
                }
            }
        }
    }
}
