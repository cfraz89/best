use std::{f32::consts::E, iter::Peekable, sync::Arc};

use proc_macro2::{
    token_stream::IntoIter, Delimiter, Group, Punct, Spacing, Span, TokenStream, TokenTree,
};
use quote::{quote, ToTokens, TokenStreamExt};

use crate::node::ChimeraMacroNode;

/// Parse a chimera notation macro template into a TemplateNode
pub fn chimera(input: TokenStream) -> TokenStream {
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
            at.unwrap()
                .error(format!("invalid token, expected {expected}, found {found}"))
                .emit();
            panic!("Failed to parse element! macro");
        }
        Err(ParseError::InvalidClose { open, close, at }) => {
            at.unwrap()
                .error(format!("invalid close tag, expected {open}, found {close}"))
                .emit();
            panic!("Failed to parse element! macro");
        }
    }
}

#[derive(thiserror::Error, Debug, Clone)]
enum ParseError {
    #[error("invalid token - expected {expected:?}, found {found:?}")]
    UnexpectedToken {
        expected: String,
        found: String,
        at: Span,
    },
    #[error("unexpected end of input - expected {expected:?}")]
    EndOfInput { expected: String },
    #[error("invalid close tag - expected {open:?}, found {close:?}")]
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

/// Parse a node, which takes the form of:
/// - an entity tag: <tag component1 component2>child</tag> or "text"
/// - an if node
fn parse_node(iter: &mut Peekable<IntoIter>) -> Result<Option<ChimeraMacroNode>, ParseError> {
    let token: Result<TokenTree, ParseError>;
    token = peek_token(iter, "<components>, #if, or text".to_string());
    match token.clone() {
        Ok(TokenTree::Punct(p)) if p.as_char() == '<' => parse_entity_tag(iter).map(Some),
        Ok(TokenTree::Punct(p)) if p.as_char() == '#' => parse_if(iter).map(Some),
        Ok(_) => parse_text(iter).map(Some),
        Err(ParseError::EndOfInput { expected: _ }) => {
            iter.next();
            Ok(None)
        }
        _ => Err(ParseError::UnexpectedToken {
            expected: "<tag or text".to_string(),
            found: token.clone()?.to_string(),
            at: token.clone()?.span(),
        }),
    }
}

fn parse_text(iter: &mut Peekable<IntoIter>) -> Result<ChimeraMacroNode, ParseError> {
    let mut text = String::new();
    loop {
        let token = peek_token(iter, "text or <".to_string())?;
        match token {
            TokenTree::Punct(p) if p.as_char() == '<' => {
                break;
            }
            t => {
                let token = take_token(iter, "text".to_string())?;
                text.push_str(&token.to_string());
            }
        }
    }
    Ok(ChimeraMacroNode::Entity {
        components: vec![quote!(chimera_rs::html::Text(#text.to_string()))],
        child_nodes: Arc::new(vec![]),
    })
}

/// Parse a  tag name
fn parse_tag_element(iter: &mut Peekable<IntoIter>) -> Result<String, ParseError> {
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

/// Peek at upcoming tokens, return success if its beginning of end tag '</'
fn peek_end_tag(iter: &mut Peekable<IntoIter>) -> Result<(), ParseError> {
    let cloned_iter = &mut iter.clone();
    parse_punct(&take_token(cloned_iter, '<'.to_string())?, '<')?;
    //Parse closing tag slash
    parse_punct(&take_token(cloned_iter, '/'.to_string())?, '/')?;
    Ok(())
}

fn capitalize(s: &str) -> String {
    format!(
        "{}{}",
        s.chars().next().unwrap().to_uppercase(),
        s.chars().skip(1).collect::<String>()
    )
}

/// Parse components of an entity node, which is a space seperated list of struct initializers
fn parse_entity_tag(iter: &mut Peekable<IntoIter>) -> Result<ChimeraMacroNode, ParseError> {
    parse_punct(&take_token(iter, '<'.to_string())?, '<')?;
    let element = parse_tag_element(iter)?;
    let primary_component = TokenTree::Ident(proc_macro2::Ident::new(
        &capitalize(&element).replace("-", "_").as_str(),
        proc_macro2::Span::call_site(),
    ));
    let mut components_tokens: Vec<TokenStream> =
        vec![quote! { Tag(#element) }, primary_component.into()];
    loop {
        if peek_self_close_tag(iter).is_ok() {
            parse_self_close_tag(iter)?;
            break;
        }
        let token = take_token(iter, "component or >".to_string())?;
        match token.clone() {
            TokenTree::Punct(p) if p.as_char() == '>' => break,
            TokenTree::Ident(_) => {
                let mut stream = TokenStream::from(token);
                //Check if its a group next, it would be a struct initializer, if so put that alongside
                let next = peek_token(iter, "next component".to_string())?;
                match next {
                    TokenTree::Group(_) => {
                        stream.append(next);
                        iter.next();
                    }
                    _ => {}
                }
                components_tokens.push(stream);
            }
            _ => {}
        }
    }
    let mut child_nodes = vec![];

    loop {
        match peek_end_tag(iter) {
            Ok(_) => break,
            Err(e @ ParseError::EndOfInput { .. }) => return Err(e),
            Err(e @ ParseError::InvalidClose { .. }) => return Err(e),
            Err(_) => {
                if let Some(node) = parse_node(iter)? {
                    child_nodes.push(node);
                }
            }
        }
    }
    //Parse closing tag opening bracket
    parse_punct(&take_token(iter, '<'.to_string())?, '<')?;
    //Parse closing tag slash
    parse_punct(&take_token(iter, '/'.to_string())?, '/')?;
    //Ensure closing tag name matches opening tag name
    let close_token = peek_token(iter, "tag name".to_string())?;
    let close_tag = parse_tag_element(iter)?;
    if close_tag != element {
        return Err(ParseError::InvalidClose {
            open: element,
            close: close_tag,
            at: close_token.span(),
        });
    }
    //Parse closing tag closing bracket
    parse_punct(&take_token(iter, '>'.to_string())?, '>')?;

    Ok(ChimeraMacroNode::Entity {
        components: components_tokens,
        child_nodes: Arc::new(child_nodes),
    })
}

fn parse_if(iter: &mut Peekable<IntoIter>) -> Result<ChimeraMacroNode, ParseError> {
    parse_punct(&take_token(iter, '#'.to_string())?, '#')?;
    let token = take_token(iter, "if".to_string())?;
    match token {
        TokenTree::Ident(i) if i.to_string() == "if" => {
            let (condition, inner_tree) = parse_if_inner(iter)?;
            let mut inner = inner_tree.stream().into_iter().peekable();
            let mut child_nodes = vec![];
            loop {
                let child = parse_node(&mut inner)?;
                if let Some(child) = child {
                    child_nodes.push(child);
                } else {
                    break;
                }
            }
            Ok(ChimeraMacroNode::If {
                condition,
                child_nodes: Arc::new(child_nodes),
            })
        }
        _ => Err(ParseError::UnexpectedToken {
            expected: "if".to_string(),
            found: token.to_string(),
            at: token.span(),
        }),
    }
}

/// Return the condition and inner tokens of an if statement
fn parse_if_inner(iter: &mut Peekable<IntoIter>) -> Result<(TokenStream, Group), ParseError> {
    let mut condition_tokens: TokenStream = TokenStream::new();
    loop {
        let token = take_token(iter, "condition, or { children }".to_string())?;
        match &token {
            TokenTree::Group(p) if p.delimiter() == Delimiter::Brace => {
                return Ok((condition_tokens, p.clone()));
            }
            _ => {
                condition_tokens.append(token);
            }
        }
    }
}
