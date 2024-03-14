use std::sync::Arc;

use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};

#[derive(Debug, Clone)]
pub enum BestMacroNode {
    Entity {
        bundle: TokenStream,
        child_nodes: Arc<Vec<BestMacroNode>>,
    },
    If {
        condition: TokenStream,
        child_nodes: Arc<Vec<BestMacroNode>>,
    },
}

/// Macrotic writing out TemplateNode -> Node
impl ToTokens for BestMacroNode {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        {
            match self {
                BestMacroNode::If {
                    condition,
                    child_nodes,
                } => {
                    let child_nodes = child_nodes.iter().map(|c| {
                        quote! {
                            Box::new(#c) as Box<dyn best::node::BestNode>
                        }
                    });
                    tokens.extend(quote! {
                        best::node::IfNode {
                            condition: || #condition,
                            child_nodes: vec![#(#child_nodes),*],
                        }
                    })
                }
                BestMacroNode::Entity {
                    bundle,
                    child_nodes,
                } => {
                    let child_nodes = child_nodes.iter().map(|c| {
                        quote! {
                            Box::new(#c) as Box<dyn best::node::BestNode>
                        }
                    });
                    tokens.extend(quote! {
                        best::node::EntityNode {
                            bundle: (#bundle),
                            child_nodes: vec![#(#child_nodes),*],
                        }
                    })
                }
            }
        }
    }
}
