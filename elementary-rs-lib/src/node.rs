use std::{collections::HashMap, future::Future, sync::Arc};

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
    Expression(String, Box<dyn Fn() -> String>),
}

pub struct HtmlElement {
    pub tag: String,
    pub attributes: HashMap<String, String>,
}

pub trait ComponentData {
    #[cfg(not(any(target_arch = "wasm32", feature = "web")))]
    fn set_server_data(&mut self, data: serde_json::Value);
    #[cfg(any(target_arch = "wasm32", feature = "web"))]
    fn get_server_data(&mut self) -> serde_json::Value;
    fn tag(&self) -> &'static str;
}

#[async_trait::async_trait]
pub trait Component: ComponentData {
    async fn view(&self) -> Node;
}

pub async fn load_server_data<
    D: serde::Serialize + serde::de::DeserializeOwned,
    F: Future<Output = D>,
>(
    component: &mut impl Component,
    load: impl Fn() -> F,
) -> D {
    cfg_if::cfg_if! {
        if #[cfg(not(any(target_arch = "wasm32", feature = "web")))] {
            let data = load().await;
            component.set_server_data(serde_json::to_value(&data).unwrap());
            data
        } else if #[cfg(any(target_arch = "wasm32", feature = "web"))] {
            let data = component.get_server_data();
            serde_json::from_value(data).unwrap()
        } else {
            unreachable!()
        }
    }
}

cfg_if::cfg_if! {
    if #[cfg(not(any(target_arch = "wasm32", feature = "web")))] {
        use std::fmt::{Write};
        use async_recursion::async_recursion;
        impl Node {
            #[async_recursion(?Send)]
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
                        if child_nodes.is_empty() {
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
                            "<{}><template shadowrootmode=\"open\">{}",
                            element.tag(),
                            element.view().await.render().await?
                        )?;
                        for child in child_nodes.iter() {
                            output.write_str(child.render().await?.as_str())?;
                        }
                        write!(output, "</{}>", element.tag())?;
                        Ok(output)
                    }
                    Node::Expression(id, exp_fn) => {
                        Ok(format!("<slot id=\"{}\">{}</slot>", id, exp_fn()))

                    }
                }
            }
        }
    }  else {
        use web_sys::{wasm_bindgen::JsValue, window};
        impl Node {
            // pub fn web_node(&self) -> Result<web_sys::Node, JsValue> {
            //     match self {
            //         Node::Text(string) => Ok(web_sys::Text::new_with_data(string)?.into()),
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
                        let result = document
                            .query_selector(format!("slot#{id}").as_str())?
                            .unwrap();
                        result.set_text_content(Some(format!("JS: {}", expr()).as_str()));
                        Ok(())
                    }
                }
            }
        }
    }
}
