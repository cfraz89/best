use std::{collections::HashMap, future::Future, pin::Pin, sync::Arc};

use crate::{
    selector::Selector,
    server_data::{SerialServerData, ServerData},
    tag::Tag,
    world::WORLD,
};
use bevy_ecs::entity::Entity;
use serde::Serialize;
use wasm_bindgen::prelude::*;
use web_sys::{window, Comment, TreeWalker};

/// https://developer.mozilla.org/en-US/docs/Web/API/Document/createTreeWalker
/// Ashamedly had to reference leptos here
const SHOW_COMMENT: u32 = 0x80;

#[derive(bevy_ecs::component::Component)]
pub enum Node {
    Text(String),
    HtmlElement {
        element: HtmlElement,
        child_nodes: Vec<Node>,
    },
    Component {
        entity: Entity,
        child_nodes: Vec<Node>,
    },
    Expression(String, Box<dyn Fn() -> String + Send + Sync>),
}

pub struct HtmlElement {
    pub tag: String,
    pub attributes: HashMap<String, String>,
}

// #[async_trait]
pub trait View {
    async fn build(&self) -> Node;
}

pub trait DynView {
    fn build(&self) -> Pin<Box<dyn Future<Output = Node> + Send + '_>>;
}

impl<T: View<build(): Send>> DynView for T {
    fn build(&self) -> Pin<Box<dyn Future<Output = Node> + Send + '_>> {
        Box::pin(View::build(self))
    }
}

#[derive(bevy_ecs::component::Component, Clone)]
pub struct AnyView(pub Arc<dyn DynView + Sync + Send>);

impl<T: View<build(): Send> + Sync + Send + 'static> From<T> for AnyView {
    fn from(component: T) -> Self {
        AnyView(Arc::new(component))
    }
}

/// Construct the view and put it into the ecs
pub async fn construct_entity_view(
    entity: &Entity,
    serial_server_data: Option<SerialServerData>,
) -> Result<(), JsValue> {
    let mut world = WORLD.write().unwrap();
    let mut entity_ref = world.entity_mut(*entity);
    let selector = entity_ref
        .get::<Selector>()
        .expect("Entity needs a selector");
    if entity_ref.get::<ServerData>().is_none() {
        if let Some(server_data) = serial_server_data.as_ref().and_then(|s| s.get(selector)) {
            entity_ref.insert(server_data);
        }
    }
    let node = entity_ref.get::<Node>();
    if node.is_none() {
        let view = entity_ref
            .get::<AnyView>()
            .expect("Entity should have a view");
        let node = view.0.build().await;
        construct_entity_view_with_node(entity, &node, serial_server_data).await?;
        entity_ref.insert(node);
    }
    Ok(())
}

#[async_recursion::async_recursion]
async fn construct_entity_view_with_node(
    entity: &Entity,
    node: &Node,
    serial_server_data: Option<SerialServerData>,
) -> Result<(), JsValue> {
    match node {
        Node::Component {
            entity,
            child_nodes,
        } => {
            for child in child_nodes.iter() {
                construct_entity_view_with_node(entity, child, serial_server_data.clone()).await?;
            }
            Ok(())
        }
        Node::HtmlElement {
            element: _,
            child_nodes,
        } => {
            for child in child_nodes.iter() {
                construct_entity_view_with_node(entity, child, serial_server_data.clone()).await?;
            }
            Ok(())
        }
        Node::Expression(id, expr) => {
            let document = window()
                .expect("No window")
                .document()
                .expect("no document");
            web_sys::console::log_1(&format!("Binding expression {:?}", id).into());
            let world = WORLD.read().unwrap();
            let entity_ref = world.entity(*entity);
            let selector = entity_ref
                .get::<Selector>()
                .expect("Entity needs a selector");
            let host_component = &document
                .query_selector(&selector.to_string())?
                .expect("id should exist");
            drop(world);
            // We walk the host component to query its child nodes
            let comments_walker =
                document.create_tree_walker_with_what_to_show(host_component, SHOW_COMMENT)?;
            replace_expression_comments(&comments_walker, id, expr)?;
            // And again for its shadow root template
            if let Some(shadow_root) = host_component.shadow_root() {
                let comments_walker = document.create_tree_walker_with_what_to_show(
                    &shadow_root.get_root_node(),
                    SHOW_COMMENT,
                )?;
                replace_expression_comments(&comments_walker, id, expr)?;
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

/// Find start and end expression comments for given expression id, empty out the inbetween and insert expression result
fn replace_expression_comments(
    comments_walker: &TreeWalker,
    id: &str,
    expr: &dyn Fn() -> String,
) -> Result<(), JsValue> {
    loop {
        web_sys::console::log_1(&format!("Iding comment {:?}", id).into());
        let start_comment = comments_walker.next_node()?;
        match start_comment {
            Some(start_comment) => {
                let start_comment = start_comment.dyn_into::<Comment>()?;
                if start_comment
                    .data()
                    .starts_with(format!("#exp:{id}").as_str())
                {
                    loop {
                        let end_comment = comments_walker.next_sibling()?;
                        match end_comment {
                            Some(end_comment) => {
                                let end_comment = end_comment.dyn_into::<Comment>()?;
                                if end_comment
                                    .data()
                                    .starts_with(format!("/exp:{id}").as_str())
                                {
                                    return replace_between_comments(
                                        start_comment,
                                        end_comment,
                                        expr,
                                    );
                                }
                            }
                            None => break,
                        }
                    }
                }
            }
            None => {
                web_sys::console::log_1(&format!("Out of comments! {:?}", id).into());
                break;
            }
        }
    }
    Ok(())
}

/// Replace all nodes between two comments with a text node containing expression results
fn replace_between_comments(
    start_comment: Comment,
    end_comment: Comment,
    expr: &dyn Fn() -> String,
) -> Result<(), JsValue> {
    let comment_parent = start_comment
        .parent_node()
        .expect("No parent node on comment");
    let mut n = start_comment
        .next_sibling()
        .expect("Nodes after start comment should have more sibilings");
    while n != **end_comment {
        let to_remove = n.to_owned();
        n = n
            .next_sibling()
            .expect("Nodes after start comment should have more sibilings");
        comment_parent.remove_child(&to_remove)?;
    }
    start_comment.after_with_str_1(format!("JS: {}", expr().as_str()).as_str())?;
    Ok(())
}

cfg_if::cfg_if! {
    if #[cfg(not(target_arch = "wasm32"))] {
        const NO_SELF_CLOSE_TAGS: &[&str] = &["slot"];

        use std::fmt::{Write};
        use async_recursion::async_recursion;
        impl Node {
            /// Render the node to a string, for server side rendering
            #[async_recursion]
            pub async fn render(&self) -> Result<String, std::fmt::Error>{
                match self {
                    Node::Text(string) => Ok(string.to_owned()),
                    Node::HtmlElement {
                        element: HtmlElement { tag, attributes },
                        child_nodes,
                    } => {
                        let mut output = String::new();
                        let attributes_string = attributes
                            .into_iter()
                            .map(|(k, v)| format!(" {}=\"{}\"", k, v))
                            .collect::<Vec<String>>()
                            .join(" ");
                        write!(output, "<{}{}", tag, attributes_string)?;
                        // Some tags don't like self_closing
                        if child_nodes.is_empty() && !NO_SELF_CLOSE_TAGS.contains(&tag.as_str()) {
                            write!(output, " />")?;
                        } else {
                            write!(output, " >")?;
                            for child in child_nodes.iter() {
                                output.write_str(child.render().await?.as_str())?;
                            }
                            write!(output, "</{}>", tag)?;
                        }
                        Ok(output)
                    }
                    Node::Component {
                        entity,
                        child_nodes,
                    } => {
                        let mut output = String::new();
                        let outer_view: Arc<dyn DynView + Send + Sync>;
                        let outer_selector_attr: String;
                        let outer_tag: String;
                        {
                        let world = WORLD.read().unwrap();
                        let entity_ref = world.entity(*entity);
                        let tag = entity_ref.get::<Tag>().expect("No tag on entity");
                        let selector = entity_ref.get::<Selector>().expect("No selector on entity");
                        let view = entity_ref.get::<AnyView>().expect("No view on enttiy");
                        outer_tag = tag.0.clone();
                        outer_view = view.0.clone();
                        outer_selector_attr = match selector {
                                Selector::Id(id) => format!("id=\"_{id}\""),
                                Selector::Class(class) => format!("class=\"_{class}\""),
                            };
                        }
                        write!(
                            output,
                            "<{} {}><template shadowrootmode=\"open\">{}</template>",
                            outer_tag,
                            outer_selector_attr,
                            outer_view.build().await.render().await?
                        )?;
                        for child in child_nodes.iter() {
                            output.write_str(child.render().await?.as_str())?;
                        }
                        write!(output, "</{}>", outer_tag)?;
                        Ok(output)
                    }
                    //We place comments around our templated expression so that we can locate it for hydration
                    Node::Expression(id, exp_fn) => {
                        Ok(format!("<!--#exp:{id}-->{}<!--/exp:{id}-->", exp_fn()))

                    }
                }
            }
        }
    }
}
