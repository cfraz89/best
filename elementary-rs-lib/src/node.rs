use std::{collections::HashMap, fmt::Debug, future::Future, pin::Pin, sync::Arc};

use crate::{context::ComponentContext, selector::Selector};
use async_trait::async_trait;
use web_sys::{wasm_bindgen::JsCast, Comment, TreeWalker};
use web_sys::{wasm_bindgen::JsValue, window};

/// https://developer.mozilla.org/en-US/docs/Web/API/Document/createTreeWalker
/// Ashamedly had to reference leptos here
const SHOW_COMMENT: u32 = 0x80;

pub enum Node {
    Text(String),
    HtmlElement {
        element: HtmlElement,
        child_nodes: Arc<Vec<Node>>,
    },
    Component {
        element: Box<dyn Component>,
        child_nodes: Arc<Vec<Node>>,
    },
    Expression(String, Box<dyn Fn() -> String + Send + Sync>),
}

impl Debug for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Node::Text(string) => write!(f, "Text: {}", string),
            Node::HtmlElement {
                element: HtmlElement { tag, attributes: _ },
                child_nodes,
            } => write!(f, "HtmlElement: {:?} {{ {:?} }}", tag, child_nodes),
            Node::Component {
                element,
                child_nodes,
            } => write!(
                f,
                "Component {:?}: {:?} {{ {:?} }}",
                element.tag(),
                element.selector(),
                child_nodes
            ),
            Node::Expression(id, _) => write!(f, "Expression: {}", id),
        }
    }
}

#[derive(Debug)]
pub struct HtmlElement {
    pub tag: String,
    pub attributes: HashMap<String, String>,
}

pub trait ComponentTag {
    fn selector(&self) -> &Selector;
    fn tag(&self) -> &'static str;
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

#[async_trait]
pub trait Component: DynView + ComponentContext + ComponentTag + Send + Sync {
    fn serialize_server_data(&self) -> ServerDataMap {
        let mut server_data_map = HashMap::new();
        server_data_map.insert(
            self.selector().to_string(),
            self.context().server_data.lock().unwrap().clone(),
        );
        serialize_node(
            self,
            self.context()
                .view
                .get()
                .expect("Can't serialize before view is built"),
            &mut server_data_map,
        );
        server_data_map
    }

    /// Bind closure expressions to wasm
    fn bind(&self) -> Result<(), JsValue> {
        web_sys::console::log_1(&format!("Binding component {:?}", self.selector()).into());
        let view = self
            .context()
            .view
            .get()
            .expect("Cannot bind before view is reified");
        bind_node(self, view)
    }

    /// Construct the view and put it into context, returning it
    async fn reified_view(
        &self,
        server_data_map: Option<&ServerDataMap>,
    ) -> Result<&Node, JsValue> {
        #[cfg(target_arch = "wasm32")]
        web_sys::console::log_1(&format!("reifying {}\n\n\n", self.selector()).into());
        let context = self.context();
        if let Some(server_data_map) = server_data_map {
            *context.server_data.lock().unwrap() = server_data_map
                .get(&self.selector().to_string())
                .expect(&format!(
                    "Server data for component {} missing",
                    self.selector().to_string()
                ))
                .clone();
        }
        if context.view.get().is_none() {
            let built_view = self.build().await;
            reify_node(&built_view, server_data_map).await?;
            context
                .view
                .set(built_view)
                .expect("Couldn't set oncelock for built_view");
        }
        Ok(context.view.get().unwrap())
    }
}

#[async_recursion::async_recursion]
async fn reify_node(
    node: &Node,
    server_data_map: Option<&'async_recursion ServerDataMap>,
) -> Result<(), JsValue> {
    match node {
        Node::Component {
            element,
            child_nodes,
        } => {
            element.reified_view(server_data_map).await;
            for child in child_nodes.iter() {
                reify_node(child, server_data_map).await;
            }
            Ok(())
        }
        Node::HtmlElement {
            element: _,
            child_nodes,
        } => {
            for child in child_nodes.iter() {
                reify_node(child, server_data_map).await;
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

fn bind_node(component: &(impl Component + ?Sized), node: &Node) -> Result<(), JsValue> {
    match node {
        Node::Text(_) => Ok(()),
        Node::HtmlElement {
            element: _,
            child_nodes,
        } => {
            for child in child_nodes.iter() {
                web_sys::console::log_1(&format!("iterating child {:?}", child).into());
                bind_node(component, child)?;
            }
            Ok(())
        }
        Node::Component {
            element,
            child_nodes,
        } => {
            element.bind()?;
            for child in child_nodes.iter() {
                web_sys::console::log_1(&format!("iterating child {:?}", child).into());
                bind_node(component, child)?;
            }
            Ok(())
        }
        Node::Expression(id, expr) => {
            let document = window()
                .expect("No window")
                .document()
                .expect("no document");
            web_sys::console::log_1(&format!("Binding expression {:?}", id).into());
            let host_component = &document
                .query_selector(&component.selector().to_string())?
                .expect("id should exist");
            // We walk the host component to query its child nodes
            let comments_walker =
                document.create_tree_walker_with_what_to_show(host_component, SHOW_COMMENT)?;
            replace_expression_comments(&comments_walker, id, expr)?;
            // And again for its shadow root template
            let shadow_root = host_component.shadow_root().expect("No shadow root");
            let comments_walker = document
                .create_tree_walker_with_what_to_show(&shadow_root.get_root_node(), SHOW_COMMENT)?;
            replace_expression_comments(&comments_walker, id, expr)?;
            Ok(())
        }
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

/// Walk the tree and push server data
fn serialize_node(component: &(impl Component + ?Sized), node: &Node, into: &mut ServerDataMap) {
    match node {
        Node::Component {
            element,
            child_nodes,
        } => {
            into.insert(
                element.selector().to_string(),
                element.context().server_data.lock().unwrap().clone(),
            );
            for child in child_nodes.iter() {
                serialize_node(component, child, into);
            }
        }
        Node::HtmlElement {
            element: _,
            child_nodes,
        } => {
            for child in child_nodes.iter() {
                serialize_node(component, child, into);
            }
        }
        _ => {}
    }
}

pub type ServerDataMap = HashMap<String, HashMap<String, serde_json::Value>>;

const NO_SELF_CLOSE_TAGS: &[&str] = &["slot"];

cfg_if::cfg_if! {
    if #[cfg(not(target_arch = "wasm32"))] {
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
                        element,
                        child_nodes,
                    } => {
                        let mut output = String::new();
                        write!(
                            output,
                            "<{} {}><template shadowrootmode=\"open\">{}</template>",
                            element.tag(),
                            match element.selector() {
                                Selector::Id(id) => format!("id=\"_{id}\""),
                                Selector::Class(class) => format!("class=\"_{class}\""),
                            },
                            element.reified_view(None).await.unwrap().render().await?
                        )?;
                        for child in child_nodes.iter() {
                            output.write_str(child.render().await?.as_str())?;
                        }
                        write!(output, "</{}>", element.tag())?;
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
