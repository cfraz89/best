use std::{collections::HashMap, future::Future, pin::Pin, sync::Arc};

use async_trait::async_trait;
use web_sys::{wasm_bindgen::JsCast, Comment, TreeWalker};

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
        element: Box<dyn DynComponent + Send + Sync>,
        child_nodes: Arc<Vec<Node>>,
    },
    Expression(String, Box<dyn Fn() -> String + Send + Sync>),
}

pub struct HtmlElement {
    pub tag: String,
    pub attributes: HashMap<String, String>,
}

// pub trait ComponentLoad {
//     #[cfg(not(target_arch = "wasm32"))]
//     fn server_load<D: serde::Serialize, F: Future<Output = D>>(
//         &self,
//         load: impl Fn() -> F,
//     ) -> impl Future<Output = D>;
//     #[cfg(target_arch = "wasm32")]
//     fn server_load<D: serde::de::DeserializeOwned, F: Future<Output = D>>(
//         &self,
//         load: impl Fn() -> F,
//     ) -> impl Future<Output = D>;
// }

pub trait ComponentTag {
    fn tag(&self) -> &'static str;
}

// #[async_trait]
pub trait Component: ComponentTag {
    async fn build(&self) -> Node;
}

pub trait DynComponent: ComponentTag {
    fn build(&self) -> Pin<Box<dyn Future<Output = Node> + Send + '_>>;
}

impl<T: Component<build(): Send>> DynComponent for T {
    fn build(&self) -> Pin<Box<dyn Future<Output = Node> + Send + '_>> {
        Box::pin(Component::build(self))
    }
}

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
                            "<{}><template shadowrootmode=\"open\">{}</template>",
                            element.tag(),
                            element.build().await.render().await?
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
    }  else {
        use web_sys::{wasm_bindgen::JsValue, window};
        impl Node {
            // pub fn web_node(&self) -> Result<web_sys::Node, JsValue> {
            //     match self {
            //         Node::Text(string) => Ok(target_arch = "wasm32"_sys::Text::new_with_data(string)?.into()),
            //         Node::HtmlElement {
            //             element: HtmlElement { tag, attributes },
            //             child_nodes,
            //         } => {
            //             let element = web_sys::window()
            //                 .expect("No window")
            //                 .document()
            //                 .expect("no document")
            //                 .create_element(tag)?;
            //             for child in child_nodes.iter() {}
            //             Ok(element.into())
            //         }
            //         Node::Component {
            //             element,
            //             child_nodes,
            //         } => {
            //             let element = web_sys::window()
            //                 .expect("No window")
            //                 .document()
            //                 .expect("no document")
            //                 .create_element(element.tag())?;
            //             Ok(element.into())
            //         }
            //         Node::Expression(uuid, exp_fn) => {
            //             let text = web_sys::Text::new_with_data(&exp_fn())?;
            //             //Todo implement signal listeners here
            //             Ok(text.into())
            //         }
            //     }
            // }

            pub fn bind(&self) -> Result<(), JsValue> {
                match self {
                    Node::Text(_) => Ok(()),
                    Node::HtmlElement {
                        element: HtmlElement { tag, attributes },
                        child_nodes,
                    } => {
                        for child in child_nodes.iter() {
                            child.bind()?;
                        }
                        Ok(())
                    }
                    Node::Component {
                        element,
                        child_nodes,
                    } => {
                        for child in child_nodes.iter() {
                            child.bind()?;
                        }
                        Ok(())
                    }
                    Node::Expression(id, expr) => {
                        let document = window()
                            .expect("No window")
                            .document()
                            .expect("no document");
                        let comments_walker = document
                          .create_tree_walker_with_what_to_show(&document.body().expect("no body"), SHOW_COMMENT)?;
                        replace_expression_comments(&comments_walker, id, expr)?;
                        Ok(())
                    }
                }
            }

        }

        /// Find start and end expression comments for given expression id, empty out the inbetween and insert expression result
        fn replace_expression_comments(comments_walker: &TreeWalker, id: &str, expr: &dyn Fn() -> String) -> Result<(), JsValue> {
            loop {
                let start_comment = comments_walker.next_node()?;
                match start_comment {
                    Some(start_comment) => {
                        let start_comment = start_comment.dyn_into::<Comment>()?;
                        if start_comment.data().starts_with(format!("#exp:{id}").as_str()) {
                            loop {
                                let end_comment = comments_walker.next_sibling()?;
                                match end_comment {
                                    Some(end_comment) => {
                                        let end_comment = end_comment.dyn_into::<Comment>()?;
                                        if end_comment.data().starts_with(format!("/exp:{id}").as_str()) {
                                            return replace_between_comments(start_comment, end_comment, expr)
                                        }
                                    }
                                    None => break
                                }
                            }
                        }
                    }
                    None => break
                }
            }
            Ok(())
        }

        /// Replace all nodes between two comments with a text node containing expression results
        fn replace_between_comments(start_comment: Comment, end_comment: Comment, expr: &dyn Fn() -> String) -> Result<(), JsValue> {
            let comment_parent = start_comment.parent_node().expect("No parent node on comment");
            let mut n = start_comment.next_sibling().expect("Nodes after start comment should have more sibilings");
            while n != **end_comment {
                let to_remove = n.to_owned();
                n = n.next_sibling().expect("Nodes after start comment should have more sibilings");
                comment_parent.remove_child(&to_remove)?;
            }
            start_comment.after_with_str_1(format!("JS: {}", expr().as_str()).as_str())?;
            Ok(())
        }

    }
}
