#![feature(proc_macro_span)]
#![feature(proc_macro_diagnostic)]
mod node;

use std::{collections::HashMap, iter::Peekable, sync::Arc};

use node::TemplateNode;
use proc_macro::{token_stream::IntoIter, Spacing, Span, TokenStream, TokenTree};
use proc_macro2::Ident;
use quote::{quote, ToTokens};
use syn::{self, parse_macro_input, punctuated::Punctuated, ImplItemFn, ItemStruct};

#[proc_macro_attribute]
pub fn server(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut item_fn = parse_macro_input!(item as ImplItemFn);
    let ident = item_fn.clone().sig.ident;
    let hidden_name = format!("_{}", ident);
    let hidden_ident = Ident::new(hidden_name.as_str(), ident.span());
    let sig = item_fn.sig.clone();
    item_fn.sig.ident = hidden_ident.clone();
    let hidden_args = item_fn
        .sig
        .inputs
        .clone()
        .into_iter()
        .skip(1)
        .collect::<Punctuated<syn::FnArg, syn::token::Comma>>();
    let await_tokens = match item_fn.sig.asyncness {
        Some(_) => quote! { .await },
        None => proc_macro2::TokenStream::new(),
    };
    quote! {
        #item_fn

        #[cfg(not(target_arch = "wasm32"))]
        #sig {
            let data = self.#hidden_ident(#hidden_args)#await_tokens;
            let mut server_data = self._context.server_data.lock().unwrap();
            server_data.insert(#hidden_name.to_string(), serde_json::to_value(&data).unwrap());
            data
        }

        //On the client, load the serialized data
        #[cfg(target_arch = "wasm32")]
        #sig {
            web_sys::console::log_1(&wasm_bindgen::JsValue::from_str(&format!("{:?}", self._context)));
            if let Some(data) = self._context.server_data.lock().unwrap().get(#hidden_name) {
                serde_json::from_value(data.clone()).expect("No server data to load!")
            } else {
                panic!("No server data")
            }
        }
    }
    .into()
}

#[proc_macro_attribute]
pub fn component(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut item_struct = parse_macro_input!(item as ItemStruct);

    if let syn::Fields::Named(ref mut fields) = item_struct.fields {
        let context_field: syn::Field = syn::parse_quote! {
            #[serde(skip)]
            pub _context: elementary_rs_lib::context::Context
        };
        let selector_field: syn::Field = syn::parse_quote! {
            pub _selector: elementary_rs_lib::selector::Selector
        };
        fields.named.push(context_field);
        fields.named.push(selector_field);
    }

    let ident = item_struct.ident.clone();
    let ident_string = format!("component-{}", ident.to_string().to_ascii_lowercase());

    quote! {
        #[cfg_attr(target_arch = "wasm32", derive(serde::Deserialize))]
        #[cfg_attr(not(target_arch = "wasm32"), derive(serde::Serialize))]
        #item_struct

        impl elementary_rs_lib::context::ComponentContext for #ident {
            fn context(&self) -> &elementary_rs_lib::context::Context {
                &self._context
            }
        }

        impl elementary_rs_lib::node::ComponentTag for #ident {
            fn selector(&self) -> &elementary_rs_lib::selector::Selector {
                &self._selector
            }

            fn tag(&self) -> &'static str {
                #ident_string
            }
        }

        impl elementary_rs_lib::node::Component for #ident {}
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

/// Take a single token and return failure if it isn't what we expected
fn take_token(iter: &mut Peekable<IntoIter>, expected: String) -> Result<TokenTree, ParseError> {
    iter.next().ok_or(ParseError::EndOfInput { expected })
}

/// Peek at a token and return failure if it isn't what we expected
fn peek_token(iter: &mut Peekable<IntoIter>, expected: String) -> Result<TokenTree, ParseError> {
    iter.peek()
        .ok_or(ParseError::EndOfInput { expected })
        .cloned()
}

/// Parse a punctuation character and return failure if it isn't what we expected
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

/// Peek at upcoming tokens, return success if its beginning of end tag '</'
fn peek_end_tag(iter: &mut Peekable<IntoIter>) -> Result<(), ParseError> {
    let cloned_iter = &mut iter.clone();
    parse_punct(&take_token(cloned_iter, '<'.to_string())?, '<')?;
    //Parse closing tag slash
    parse_punct(&take_token(cloned_iter, '/'.to_string())?, '/')?;
    Ok(())
}

/// Peek at upcoming tokens, return success if its self closing tag '/>'
fn peek_self_close_tag(iter: &mut Peekable<IntoIter>) -> Result<(), ParseError> {
    let cloned_iter = &mut iter.clone();
    parse_punct(&take_token(cloned_iter, '/'.to_string())?, '/')?;
    //Parse closing tag slash
    parse_punct(&take_token(cloned_iter, '>'.to_string())?, '>')?;
    Ok(())
}

/// Parse upcoming tokens, return success if its self closing tag '/>'
fn parse_self_close_tag(iter: &mut Peekable<IntoIter>) -> Result<(), ParseError> {
    parse_punct(&take_token(iter, '/'.to_string())?, '/')?;
    //Parse closing tag slash
    parse_punct(&take_token(iter, '>'.to_string())?, '>')?;
    Ok(())
}

/// Parse a  tag name, which may be a html element or a custom element
fn parse_tag_name(iter: &mut Peekable<IntoIter>) -> Result<String, ParseError> {
    let mut tag: Option<String> = None;
    let mut joint_to_continue_name = false;
    loop {
        let input = peek_token(iter, "tag name".to_string())?;
        match (input, &mut tag) {
            (TokenTree::Ident(i), None) => {
                tag = Some(i.to_string());
                joint_to_continue_name = true;
                take_token(iter, "tag name".to_string())?;
                Ok(())
            }
            (TokenTree::Ident(i), Some(t)) => {
                //We got an ident instead of a joint, we've gone past the tag name
                if joint_to_continue_name {
                    break;
                }
                t.push_str(&i.to_string());
                Ok(())
            }
            (TokenTree::Punct(p), Some(t))
                if p.as_char() == '-' && p.spacing() == Spacing::Joint =>
            {
                t.push_str("-");
                joint_to_continue_name = false;
                Ok(())
            }
            (TokenTree::Punct(p), Some(_t)) if p.as_char() == '/' || p.as_char() == '>' => {
                break;
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
    //Treat it as a text node until we hit a '<' or '{'
    let mut token: TokenTree;
    loop {
        token = peek_token(iter, "text".to_string())?;
        match &token {
            TokenTree::Punct(p) if p.as_char() == '<' => {
                break;
            }
            TokenTree::Group(g) if g.delimiter() == proc_macro::Delimiter::Brace => {
                take_token(iter, "group".to_string())?;
                return Ok(TemplateNode::Expression(g.stream().into()));
            }
            _ => {}
        }
        let token = take_token(iter, "text".to_string())?;
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
            Element::Html {
                element,
                child_nodes,
            } => TemplateNode::HtmlElement {
                element,
                child_nodes,
            },
            Element::Component {
                element,
                child_nodes,
            } => TemplateNode::ComponentElement {
                element,
                child_nodes,
            },
        })
    }
}

enum Element {
    Html {
        element: node::HtmlElement,
        child_nodes: Arc<Vec<TemplateNode>>,
    },
    Component {
        element: node::ComponentElement,
        child_nodes: Arc<Vec<TemplateNode>>,
    },
}

/// Parse an element tag, which may be a html element or a custom element
/// If its a custom element, we just keep the raw tokens for each attribute
fn parse_element(iter: &mut Peekable<IntoIter>) -> Result<Element, ParseError> {
    //Parse opening tag opening bracket
    parse_punct(&take_token(iter, '<'.to_string())?, '<')?;
    //Parse tag name
    let tag = parse_tag_name(iter)?;

    if peek_self_close_tag(iter).is_ok() {
        parse_self_close_tag(iter)?;
        return get_element(tag, Arc::new(Vec::new()));
    }

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

    get_element(tag, Arc::new(child_nodes))
}

fn get_element(tag: String, child_nodes: Arc<Vec<TemplateNode>>) -> Result<Element, ParseError> {
    //If its an uppercase tag, its a custom element
    if tag.chars().next().unwrap().is_uppercase() {
        Ok(Element::Component {
            element: node::ComponentElement {
                name: tag,
                //TODO convert the token stream into the hashmap
                properties: HashMap::new(),
            },
            child_nodes,
        })
    } else {
        Ok(Element::Html {
            element: node::HtmlElement {
                tag,
                attributes: HashMap::new(),
            },
            child_nodes,
        })
    }
}
