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

/// Parse an attribute
fn parse_attribute(
    iter: &mut Peekable<IntoIter>,
) -> Result<(String, Span, TokenStream), ParseError> {
    let mut attribute_name: Option<String> = None;
    let mut span: Option<Span> = None;
    loop {
        let input = peek_token(iter, "attribute name".to_string())?;
        match (input, &mut attribute_name) {
            (TokenTree::Ident(i), None) => {
                attribute_name = Some(i.to_string());
                span = Some(i.span());
                take_token(iter, "attribute name".to_string())?;
                Ok(())
            }
            (TokenTree::Ident(i), Some(t)) => {
                t.push_str(&i.to_string());
                span = Some(i.span());
                Ok(())
            }
            (TokenTree::Punct(p), Some(t))
                if p.as_char() == '-' && p.spacing() == Spacing::Joint =>
            {
                span = Some(p.span());
                t.push_str("-");
                Ok(())
            }
            (TokenTree::Punct(p), Some(_t)) if p.as_char() == '=' => {
                take_token(iter, "attribute asignment (=)".to_string())?;
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
    match (attribute_name, span) {
        (Some(t), Some(span)) => {
            let value = take_token(iter, "attribute value".to_string())?;
            match value.clone() {
                TokenTree::Literal(_) => Ok((t, span, value.into())),
                v => Err(ParseError::UnexpectedToken {
                    expected: "attribute value literal".to_string(),
                    found: v.to_string(),
                    at: v.span(),
                }),
            }
        }
        (_, _) => Err(ParseError::EndOfInput {
            expected: "tag name".to_string(),
        }),
    }
}

/// Parse a html-like macro template into a TemplateNode
fn parse_node(
    iter: &mut Peekable<IntoIter>,
    world_identifier: &Ident,
) -> Result<Option<TemplateNode>, ParseError> {
    let mut token: TokenTree;
    //Decide wether we're parsing this node as text or an expression first
    token = peek_token(iter, "text".to_string())?;
    match &token {
        TokenTree::Group(g) if g.delimiter() == proc_macro::Delimiter::Brace => {
            take_token(iter, "group".to_string())?;
            return Ok(Some(TemplateNode::Expression(g.stream().into())));
        }
        _ => {}
    };
    //Not a group/expression, so read text as long as we can
    let mut text = Option::<String>::None;
    //Treat it as a text node until we hit a '<' or '{'
    loop {
        token = peek_token(iter, "text".to_string())?;
        match &token {
            TokenTree::Punct(p) if p.as_char() == '<' => {
                break;
            }
            TokenTree::Group(g) if g.delimiter() == proc_macro::Delimiter::Brace => {
                break;
            }
            _ => {}
        }
        let token = take_token(iter, "text".to_string())?;
        if let Some(ref mut t) = text {
            t.push_str(&token.to_string());
        } else {
            println!("Set text: {:?}", token);
            text = Some(token.to_string());
        }
    }
    println!("Text: {:?}", text);
    if let Some(t) = text {
        println!("Got text: {t}");
        Ok(Some(TemplateNode::Text(t)))
    } else {
        Ok(Some(match parse_element(iter, world_identifier)? {
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
                world_identifier: proc_macro2::Ident::new(
                    &world_identifier.to_string(),
                    world_identifier.span().into(),
                ),
            },
        }))
    }
}

enum Element {
    Html {
        element: template_node::HtmlElement,
        child_nodes: Arc<Vec<TemplateNode>>,
    },
    Component {
        element: template_node::ComponentElement,
        child_nodes: Arc<Vec<TemplateNode>>,
    },
}

/// Parse an element tag, which may be a html element or a custom element
/// If its a custom element, we just keep the raw tokens for each attribute
fn parse_element(
    iter: &mut Peekable<IntoIter>,
    world_identifier: &Ident,
) -> Result<Element, ParseError> {
    //Parse opening tag opening bracket
    parse_punct(&take_token(iter, '<'.to_string())?, '<')?;
    //Parse tag name
    let tag = parse_tag_name(iter)?;

    //Pare attributes
    let mut attributes = Vec::<(String, Span, TokenStream)>::new();
    while peek_ident(iter).is_ok() {
        attributes.push(parse_attribute(iter)?);
    }

    if peek_self_close_tag(iter).is_ok() {
        parse_self_close_tag(iter)?;
        return get_element(tag, attributes, Arc::new(Vec::new()));
    }

    //Parse opening tag closing bracket
    parse_punct(&take_token(iter, '>'.to_string())?, '>')?;
    //Parse child if it exists
    let mut child_nodes = Vec::<TemplateNode>::new();

    while peek_end_tag(iter).is_err() {
        if let Some(node) = parse_node(iter, world_identifier)? {
            child_nodes.push(node);
        }
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

    get_element(
        tag,
        attributes,
        Arc::new(child_nodes).tap(|n| println!("child nodes{:?}", n)),
    )
}

/// Return our TemplateNode element given tag, attributes and child nodes
fn get_element(
    tag: String,
    attributes: Vec<(String, Span, TokenStream)>,
    child_nodes: Arc<Vec<TemplateNode>>,
) -> Result<Element, ParseError> {
    //If its an uppercase tag, its a custom element
    if tag.chars().next().unwrap().is_uppercase() {
        let properties: Vec<(proc_macro2::Ident, proc_macro2::TokenStream)> = attributes
            .into_iter()
            .map(|(name, span, v)| (proc_macro2::Ident::new(&name, span.into()), v.into()))
            .collect();
        Ok(Element::Component {
            element: template_node::ComponentElement {
                name: tag,
                //TODO convert the token stream into the hashmap
                properties,
            },
            child_nodes,
        })
    } else {
        //Parse all the strings in our attribute tokenstreams
        let attributes = attributes
            .into_iter()
            .map(
                |(name, span, v)| match syn::parse::<syn::LitStr>(v.clone()) {
                    Ok(s) => Ok((name, s.value())),
                    Err(e) => Err(ParseError::UnexpectedToken {
                        expected: "string literal".to_string(),
                        found: v.to_string(),
                        at: span,
                    }),
                },
            )
            .collect::<Result<Vec<(String, String)>, ParseError>>()?;
        Ok(Element::Html {
            element: template_node::HtmlElement { tag, attributes },
            child_nodes,
        })
    }
}
