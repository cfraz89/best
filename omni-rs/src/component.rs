use std::{borrow::Borrow, collections::HashMap, fmt::Write, sync::Arc};

pub enum Element<'a> {
    Html(HtmlElement<'a>),
    Component(Box<dyn Component>),
}

pub enum ElementChild<'a> {
    Elements(Vec<Element<'a>>),
    Text(String),
}

pub struct HtmlElement<'a> {
    pub tag: &'a str,
    pub attributes: HashMap<String, String>,
    pub child: Arc<ElementChild<'a>>,
}

pub trait Component {
    fn template(&self) -> Element;
}

pub trait Renderable {
    fn render(&self) -> String;
}

impl Renderable for Element<'_> {
    fn render(&self) -> String {
        let mut str = String::new();
        match self {
            Element::Html(HtmlElement {
                tag,
                attributes,
                child,
            }) => {
                let attributes_string = attributes
                    .into_iter()
                    .map(|(k, v)| format!(" {}=\"{}\"", k, v))
                    .collect::<Vec<String>>()
                    .join(" ");
                write!(&mut str, "<{}{}>", tag, attributes_string)
                    .expect("couldn't write opening tag");
                match child.borrow() {
                    ElementChild::Text(text) => {
                        str.write_str(text).expect("couldn't write text");
                    }
                    ElementChild::Elements(children) => {
                        for child in children {
                            str.write_str(&child.render()).expect("couldn't child");
                        }
                    }
                }
                write!(&mut str, "</{}>", tag).expect("couldn't write closing tag");
            }
            Element::Component(component) => str
                .write_str(&component.template().render())
                .expect("couldn't write children"),
        }
        str
    }
}

impl Renderable for ElementChild<'_> {
    fn render(&self) -> String {
        match self {
            ElementChild::Text(text) => text.clone(),
            ElementChild::Elements(children) => children
                .into_iter()
                .map(|child| child.render())
                .collect::<Vec<String>>()
                .join(""),
        }
    }
}

pub struct Signal {}
