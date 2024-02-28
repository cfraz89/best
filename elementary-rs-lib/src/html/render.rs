use super::node::{Element, Node};
use std::fmt::Write;

pub enum RenderedNode {
    Single(String),
    Parent { open: String, close: String },
}

static NO_SELF_CLOSE_TAGS: [&str; 3] = ["script", "style", "template"];

///Render a node instance, will be partial if component templates don't exist yet
pub fn render_tag(node: &Node, has_children: bool) -> Result<RenderedNode, std::fmt::Error> {
    match node {
        Node::Text(string) => Ok(RenderedNode::Single(string.to_owned())),
        Node::Element(Element { tag, attributes }) => {
            let mut open = String::new();
            let attributes_string = attributes
                .into_iter()
                .map(|(k, v)| format!(" {}=\"{}\"", k, v))
                .collect::<Vec<String>>()
                .join(" ");
            write!(open, "<{}{}", tag, attributes_string)?;
            // Some tags don't like self_closing
            if !has_children && !NO_SELF_CLOSE_TAGS.contains(&tag.as_str()) {
                write!(open, " />")?;
                return Ok(RenderedNode::Single(open));
            } else {
                write!(open, " >")?;
            }
            Ok(RenderedNode::Parent {
                open,
                close: format!("</{}>", tag),
            })
        }
    }
}
