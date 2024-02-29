use std::sync::Arc;

use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};

#[derive(Debug, Clone)]
pub struct EntityNode {
    pub builder_identifier: Ident,
    pub components: TokenStream,
    pub child_nodes: Arc<Vec<EntityNode>>,
}

/// Macrotic writing out TemplateNode -> Node
impl ToTokens for EntityNode {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        {
            let builder_identifier = self.builder_identifier.clone();
            let components = self.components.clone();
            let child_nodes = self.child_nodes.clone();
            let children: TokenStream = if child_nodes.len() > 0 {
                quote! {
                    .with_children(|builder| {
                        #(#child_nodes),*
                    })
                }
            } else {
                TokenStream::new()
            }
            .into();
            tokens.extend(quote! {
                #builder_identifier.spawn((#components))#children;
            })
        }
    }
}
