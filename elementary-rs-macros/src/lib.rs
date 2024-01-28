#![feature(proc_macro_span)]
#![feature(proc_macro_diagnostic)]
#![feature(lazy_cell)]
mod node;

use std::{
    collections::HashMap,
    iter::Peekable,
    sync::{Arc, LazyLock, Mutex},
};

use node::{HtmlElement, TemplateNode};
use proc_macro::{token_stream::IntoIter, Span, TokenStream, TokenTree};
use quote::{quote, ToTokens};
use syn::{self, meta::ParseNestedMeta, parse_macro_input, DeriveInput, LitStr};

static REGISTERED_COMPONENTS: LazyLock<Mutex<HashMap<String, String>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

#[proc_macro_derive(CustomElement, attributes(custom_element))]
pub fn derive_component(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);
    let mut tag = Option::None;
    input.attrs.iter().for_each(|attr| {
        if attr.path().is_ident("custom_element") {
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("tag") {
                    tag = Some(meta.value()?.parse::<LitStr>()?.value());
                    return Ok(());
                }
                Err(meta.error("expected `tag` or `tag = \"...\"`"))
            })
            .unwrap();
        }
    });
    let ident = input.ident;
    let ident_string = format!("component-{}", ident.to_string().to_ascii_lowercase());
    let final_tag = tag.unwrap_or(ident.to_string());
    REGISTERED_COMPONENTS
        .lock()
        .unwrap()
        .insert(final_tag.clone(), ident.to_string());
    println!(
        "Registered component: {:?} -> {:?}",
        final_tag,
        ident.clone()
    );

    // println!("{:?}", input);
    quote! {
        impl elementary_rs_lib::node::CustomElement for #ident {
            fn tag(&self) -> &'static str {
                #ident_string
            }
        }
    }
    .into()
}

#[derive(thiserror::Error, Debug)]
enum ParseError {
    #[error("invalid token - expected {expected:?}, found {found:?}")]
    UnexpectedToken {
        expected: String,
        found: String,
        at: Span,
    },
    #[error("unexpected end of input - expected {expected:?}")]
    EndOfInput { expected: String },
    #[error("unexpected closing tag {close:?}, expected {open:?}")]
    InvalidClose {
        open: String,
        close: String,
        at: Span,
    },
}

fn take_token(iter: &mut Peekable<IntoIter>, expected: String) -> Result<TokenTree, ParseError> {
    iter.next().ok_or(ParseError::EndOfInput { expected })
}

fn peek_token(iter: &mut Peekable<IntoIter>, expected: String) -> Result<TokenTree, ParseError> {
    iter.peek()
        .ok_or(ParseError::EndOfInput { expected })
        .cloned()
}

fn parse_punct(input: &TokenTree, punct: char) -> Result<(), ParseError> {
    match input {
        TokenTree::Punct(p) => {
            if p.as_char() == punct {
                Ok(())
            } else {
                Err(ParseError::UnexpectedToken {
                    expected: punct.to_string(),
                    found: p.to_string(),
                    at: p.span(),
                })
            }
        }
        t => Err(ParseError::UnexpectedToken {
            expected: punct.to_string(),
            found: t.to_string(),
            at: t.span(),
        }),
    }
}

fn peek_end_tag(iter: &mut Peekable<IntoIter>) -> Result<(), ParseError> {
    let cloned_iter = &mut iter.clone();
    parse_punct(&take_token(cloned_iter, '<'.to_string())?, '<')?;
    //Parse closing tag slash
    parse_punct(&take_token(cloned_iter, '/'.to_string())?, '/')?;
    Ok(())
}

fn parse_tag_name(iter: &mut Peekable<IntoIter>) -> Result<String, ParseError> {
    let mut tag: Option<String> = None;
    while parse_punct(&peek_token(iter, "tag name".to_string())?, '>').is_err() {
        let input = take_token(iter, "tag name".to_string())?;
        match (input, &mut tag) {
            (TokenTree::Ident(i), None) => {
                tag = Some(i.to_string());
                Ok(())
            }
            (TokenTree::Ident(i), Some(t)) => {
                t.push_str(&i.to_string());
                Ok(())
            }
            (TokenTree::Punct(p), Some(t)) if p.as_char() == '-' => {
                t.push_str("-");
                Ok(())
            }

            (TokenTree::Punct(p), None) => Err(ParseError::UnexpectedToken {
                expected: "identifier".to_string(),
                found: p.to_string(),
                at: p.span(),
            }),
            (t, _) => Err(ParseError::UnexpectedToken {
                expected: "identifier".to_string(),
                found: t.to_string(),
                at: t.span(),
            }),
        }?;
    }
    match tag {
        Some(t) => Ok(t),
        None => Err(ParseError::EndOfInput {
            expected: "tag name".to_string(),
        }),
    }
}

/// Parse a html-like template into a TemplateNode. Then emit its token stream, which will generate a Node
#[proc_macro]
pub fn node(input: TokenStream) -> TokenStream {
    let iter = &mut input.into_iter().peekable();
    match parse_node(iter) {
        Ok(node) => {
            println!("{:?}", node);
            node.to_token_stream().into()
        }
        Err(ParseError::EndOfInput { expected }) => {
            panic!("Unexpected end of input, expected {}", expected)
        }
        Err(ParseError::UnexpectedToken {
            expected,
            found,
            at,
        }) => {
            at.error(format!("invalid token, expected {expected}, found {found}"))
                .emit();
            panic!("Failed to parse element! macro");
        }
        Err(ParseError::InvalidClose { open, close, at }) => {
            at.error(format!(
                "invalid closing tag, expected {open}, found {close}"
            ))
            .emit();
            panic!("Failed to parse element! macro");
        }
    }
}

/// Parse a html-like template macro template, which when generated will render it out to a string
#[proc_macro]
pub fn render_node(input: TokenStream) -> TokenStream {
    let stream: proc_macro2::TokenStream = node(input).into();
    quote! {
        <elementary_rs_lib::node::Node as elementary_rs_lib::node::Renderable>::render(&
         #stream
        )
    }
    .into()
}

/// Parse a html-like macro template into a TemplateNode
fn parse_node(iter: &mut Peekable<IntoIter>) -> Result<TemplateNode, ParseError> {
    let mut text = Option::<String>::None;
    while parse_punct(&peek_token(iter, "text".to_string())?, '<').is_err() {
        let token = iter.next().ok_or(ParseError::EndOfInput {
            expected: "text".to_string(),
        })?;
        if let Some(ref mut t) = text {
            t.push_str(&token.to_string());
        } else {
            text = Some(token.to_string());
        }
    }
    if let Some(t) = text {
        Ok(TemplateNode::Text(t))
    } else {
        Ok(match parse_element(iter)? {
            Element::Html(html_element) => TemplateNode::HtmlElement(html_element),
            Element::Component(component_element) => {
                TemplateNode::ComponentElement(component_element)
            }
        })
    }
}

enum Element {
    Html(node::HtmlElement),
    Component(node::ComponentElement),
}

/// Parse an element tag, which may be a html element or a custom element
/// If its a custom element, we just keep the raw tokens for each attribute
fn parse_element(iter: &mut Peekable<IntoIter>) -> Result<Element, ParseError> {
    //Parse opening tag opening bracket
    parse_punct(&take_token(iter, '<'.to_string())?, '<')?;
    //Parse tag name
    let tag = parse_tag_name(iter)?;
    //Parse opening tag closing bracket
    parse_punct(&take_token(iter, '>'.to_string())?, '>')?;
    //Parse child if it exists
    let mut child_nodes = Vec::<TemplateNode>::new();

    while peek_end_tag(iter).is_err() {
        let node = parse_node(iter)?;
        child_nodes.push(node);
    }
    //Parse closing tag opening bracket
    parse_punct(&take_token(iter, '<'.to_string())?, '<')?;
    //Parse closing tag slash
    parse_punct(&take_token(iter, '/'.to_string())?, '/')?;
    //Ensure closing tag name matches opening tag name
    let close_token = peek_token(iter, "tag name".to_string())?;
    let close_tag = parse_tag_name(iter)?;
    if close_tag != tag {
        return Err(ParseError::InvalidClose {
            open: tag,
            close: close_tag,
            at: close_token.span(),
        });
    }
    //Parse closing tag closing bracket
    parse_punct(&take_token(iter, '>'.to_string())?, '>')?;

    if tag.chars().next().unwrap().is_uppercase() {
        Ok(Element::Component(node::ComponentElement {
            name: tag,
            //TODO convert the token stream into the hashmap
            properties: HashMap::from([(
                syn::Ident::new("child_nodes", proc_macro2::Span::call_site()),
                quote! {
                    Arc::new(vec![#(#child_nodes),*])
                },
            )]),
        }))
    } else {
        Ok(Element::Html(node::HtmlElement {
            tag,
            attributes: HashMap::new(),
            child_nodes: Arc::new(child_nodes),
        }))
    }
}
