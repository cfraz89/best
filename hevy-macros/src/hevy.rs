use std::{iter::Peekable, sync::Arc};

use proc_macro2::{token_stream::IntoIter, Delimiter, Ident, Punct, Span, TokenStream, TokenTree};
use quote::{quote, ToTokens, TokenStreamExt};

use crate::entity_node::EntityNode;

/// Parse a hevy notation macro template into a TemplateNode
pub fn hevy(input: TokenStream) -> TokenStream {
    let iter = &mut input.into_iter().peekable();
    let commands_identifier = match take_token(iter, "commands".to_string())
        .expect("Couldn't read commands identifier")
    {
        TokenTree::Ident(i) => i,
        t => {
            t.span()
                .unwrap()
                .error(format!(
                    "invalid token, expected commands variable identifier, found {t}"
                ))
                .emit();
            panic!("Failed to parse ecn! macro");
        }
    };
    parse_punct(
        &take_token(iter, ','.to_string()).expect("Expecting comma"),
        ',',
    )
    .expect("Couldn't read comma");

    match parse_entity(commands_identifier, iter) {
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

/// Parse an entity node, which takes the form <component1 component2> { children...} or "text"
fn parse_entity(
    builder_identifier: Ident,
    iter: &mut Peekable<IntoIter>,
) -> Result<Option<EntityNode>, ParseError> {
    let token: Result<TokenTree, ParseError>;
    token = take_token(iter, "<components> or text string".to_string());
    match token.clone() {
        Ok(TokenTree::Punct(p)) if p.as_char() == '<' => {
            parse_components(builder_identifier, iter).map(Some)
        }
        Ok(TokenTree::Literal(_)) => {
            let lit: TokenStream = token?.into();
            Ok(Some(EntityNode {
                builder_identifier,
                components: quote!(Text(#lit)),
                child_nodes: Arc::new(vec![]),
            }))
        }
        Err(ParseError::EndOfInput { expected: _ }) => Ok(None),
        _ => Err(ParseError::UnexpectedToken {
            expected: "<components...> or text".to_string(),
            found: token.clone()?.to_string(),
            at: token.clone()?.span(),
        }),
    }
}

/// Parse components of an entity node, which is a space seperated list of struct initializers
fn parse_components(
    builder_identifier: Ident,
    iter: &mut Peekable<IntoIter>,
) -> Result<EntityNode, ParseError> {
    let mut tokens: TokenStream = TokenStream::new();
    loop {
        let token = take_token(iter, "component or >".to_string())?;
        match token.clone() {
            TokenTree::Punct(p) if p.as_char() == '>' => break,
            TokenTree::Ident(_) => {
                tokens.append(token);
                //Check if its another ident next, if so insert a comma
                let next = peek_token(iter, "next component".to_string())?;
                match next {
                    TokenTree::Ident(_) => {
                        tokens.append(TokenTree::Punct(Punct::new(
                            ',',
                            proc_macro2::Spacing::Joint,
                        )));
                    }
                    _ => {}
                }
            }
            TokenTree::Group(_) => {
                tokens.append(token);
                //Check if its another ident next, if so insert a comma
                let next = peek_token(iter, "next component".to_string())?;
                match next {
                    TokenTree::Ident(_) => {
                        tokens.append(TokenTree::Punct(Punct::new(
                            ',',
                            proc_macro2::Spacing::Joint,
                        )));
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
    let mut child_nodes = vec![];
    let next = peek_token(iter, "{} or <".to_string());
    match next.clone() {
        Ok(TokenTree::Group(ref g)) if g.delimiter() == Delimiter::Brace => {
            let mut inner = g.stream().into_iter().peekable();
            loop {
                let child = parse_entity(Ident::new("builder", next.clone()?.span()), &mut inner)?;
                if let Some(child) = child {
                    child_nodes.push(child);
                } else {
                    break;
                }
            }
            //Advance the outer iterator to go over the group
            iter.next();
        }
        _ => {}
    }
    Ok(EntityNode {
        builder_identifier,
        components: tokens,
        child_nodes: Arc::new(child_nodes),
    })
}
