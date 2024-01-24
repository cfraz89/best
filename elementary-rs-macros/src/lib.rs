#![feature(proc_macro_span)]
#![feature(proc_macro_diagnostic)]
use std::iter::Peekable;

use elementary_rs_lib::node::{HtmlElement, Node};
use proc_macro::{token_stream::IntoIter, Span, TokenStream, TokenTree};
use quote::{quote, ToTokens};
use syn::{self, meta::ParseNestedMeta, parse::Parse, parse_macro_input, LitStr};

// ----- #(component(name)) attribute macro -----
#[derive(Debug)]
struct ComponentAttributes {
    tag: String,
}

impl ComponentAttributes {
    fn parse(&mut self, meta: ParseNestedMeta) -> syn::Result<()> {
        if meta.path.is_ident("tag") {
            self.tag = meta.value()?.parse::<LitStr>()?.value();
            Ok(())
        } else {
            unimplemented!()
        }
    }
}

#[proc_macro_attribute]
pub fn component(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the list of variables the user wanted to print.
    // let mut args = parse_macro_input!(attr as ComponentAttributes);
    let mut component_attributes = ComponentAttributes {
        tag: String::from(""),
    };
    let component_parser = syn::meta::parser(|meta| component_attributes.parse(meta));
    parse_macro_input!(attr with component_parser);

    item
}
// -----------

// ----- element! proc macro -----

// enum Node {
//     HtmlElement(HtmlElement),
//     Text(String),
// }

// struct HtmlElement {
//     tag: String,
//     children: Vec<Node>,
// }

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

/// Parse a html-like template into a Node
#[proc_macro]
pub fn node(input: TokenStream) -> TokenStream {
    let iter = &mut input.into_iter().peekable();
    match parse_node(iter) {
        Ok(node) => node.to_token_stream().into(),
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

/// Parse a html-like template, then render it out to a string
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

fn parse_node(iter: &mut Peekable<IntoIter>) -> Result<Node, ParseError> {
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
        Ok(Node::Text(t))
    } else {
        let element = parse_html_element(iter)?;
        Ok(Node::HtmlElement(element))
    }
}

fn parse_html_element(iter: &mut Peekable<IntoIter>) -> Result<HtmlElement, ParseError> {
    //Parse opening tag opening bracket
    parse_punct(&take_token(iter, '<'.to_string())?, '<')?;
    //Parse tag name
    let tag = parse_tag_name(iter)?;
    //Parse opening tag closing bracket
    parse_punct(&take_token(iter, '>'.to_string())?, '>')?;
    //Parse child if it exists
    let mut child_nodes = Vec::<Node>::new();

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

    Ok(HtmlElement {
        tag: tag.into(),
        child_nodes: child_nodes.into(),
        attributes: Default::default(),
    })
}
