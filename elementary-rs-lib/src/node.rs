use std::{collections::HashMap, future::Future, sync::Arc};

use async_trait::async_trait;
use web_sys::{console, wasm_bindgen::JsCast};

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
        element: Box<dyn Component + Send + Sync>,
        child_nodes: Arc<Vec<Node>>,
    },
    Expression(String, Box<dyn Fn() -> String + Send + Sync>),
}

pub struct HtmlElement {
    pub tag: String,
    pub attributes: HashMap<String, String>,
}

pub trait ComponentLoad {
    #[cfg(not(target_arch = "wasm32"))]
    fn server_load<D: serde::Serialize, F: Future<Output = D>>(
        &self,
        load: impl Fn() -> F,
    ) -> impl Future<Output = D>;
    #[cfg(target_arch = "wasm32")]
    fn server_load<D: serde::de::DeserializeOwned, F: Future<Output = D>>(
        &self,
        load: impl Fn() -> F,
    ) -> impl Future<Output = D>;
}

pub trait ComponentTag {
    fn tag(&self) -> &'static str;
}

#[async_trait]
pub trait Component: ComponentTag {
    async fn view(&self) -> Node;
}
//Object-safe component
// https://rust-lang.github.io/async-fundamentals-initiative/evaluation/case-studies/builder-provider-api.html#dynamic-dispatch-behind-the-api
// pub trait ComponentDyn {
//     #[cfg(not(any(target_arch = "wasm32", feature = "web")))]
//     fn tag(&self) -> &'static str;
//     fn view(&self) -> Pin<Box<dyn Future<Output = Node>>>;
// }

// impl<T: Component> ComponentDyn for T {
//     #[cfg(not(any(target_arch = "wasm32", feature = "web")))]
//     fn tag(&self) -> &'static str {
//         T::tag(self)
//     }
//     fn view(&self) -> Pin<Box<dyn Future<Output = Node>>> {
//         Box::pin(T::view(self))
//     }
// }

const NO_SELF_CLOSE_TAGS: &[&str] = &["slot"];

cfg_if::cfg_if! {
    if #[cfg(not(target_arch = "wasm32"))] {
        use std::fmt::{Write};
        use async_recursion::async_recursion;
        impl Node {
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
                            element.view().await.render().await?
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
                        let comments = document
                          .create_tree_walker_with_what_to_show(&document.body().expect("no body"), SHOW_COMMENT)?;
                        'start: loop {
                            let start_comment = comments.next_node()?;
                            match start_comment {
                                Some(start_comment) => {
                                    let start_comment = start_comment.dyn_into::<web_sys::Comment>()?;
                                    if start_comment.data().starts_with(format!("#exp:{id}").as_str()) {
                                        let comment_parent = start_comment.parent_node().expect("No parent node on comment");
                                        'end: loop {
                                            let end_comment = comments.next_sibling()?;
                                            match end_comment {
                                                Some(end_comment) => {
                                                    let end_comment = end_comment.dyn_into::<web_sys::Comment>()?;
                                                    if end_comment.data().starts_with(format!("/exp:{id}").as_str()) {

                                                        let mut n = start_comment.next_sibling().expect("Nodes after start comment should have more sibilings");
                                                        while n != **end_comment {
                                                            let to_remove = n.to_owned();
                                                            n = n.next_sibling().expect("Nodes after start comment should have more sibilings");
                                                            comment_parent.remove_child(&to_remove);
                                                        }
                                                        start_comment.after_with_str_1(format!("JS: {}", expr().as_str()).as_str())?;
                                                        break 'start;
                                                    }
                                                }
                                                None => break 'end
                                            }
                                        }
                                    }
                                }
                                None => break
                            }
                        }
                        Ok(())
                    }
                }
            }
        }
    }
}
