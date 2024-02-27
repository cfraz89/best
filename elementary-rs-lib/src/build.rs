use crate::components::{AnyWebComponent, Tag, Template, WebComponentChildren};
use bevy::tasks::{block_on, futures_lite::future, IoTaskPool, Task};
use bevy_ecs::prelude::*;
use either::{Either, Left, Right};
use std::ops::Deref;

impl Deref for Template {
    type Target = NodeRef;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// #[derive(Component)]
// pub struct Markup(EitherStringResult);

cfg_if::cfg_if! {
  if #[cfg(not(target_arch = "wasm32"))] {
    use std::fmt::{Write};
    use crate::node::*;

    const NO_SELF_CLOSE_TAGS: &[&str] = &["slot"];

    //system
    // pub async fn render_page(
    //     world: &World,
    //     query: Query<(Entity, &Tag, &AnyWebComponent), With<Page>>,
    // ) -> String {
    // }

    // pub fn render_component_instance(world: &mut World, entity: Entity, tag: &Tag, child_nodes: &Vec<NodeRef>) -> Result<String, std::fmt::Error> {
    //     if let Some(task) = render_component {
    //         if let Some(output) = block_on(future::poll_once(&mut task.0)) {
    //             output
    //         } else {
    //             Ok(format!("<{}><template shadowrootmode=\"open\">", tag.0))
    //         }
    //     }
    //     else {
    //     }
    // }

    //Render components instance, will be partial if template doesn't exist yet
    pub fn render_component_instance(world: &World, entity: Entity) -> Result<Either<String, String>, std::fmt::Error> {
        let entity_ref = world.entity(entity);
        let tag = entity_ref.get::<Tag>().expect("Entity needs a tag").clone();
        let children = entity_ref.get::<WebComponentChildren>();
        let template = entity_ref.get::<Template>();
        let mut output = String::new();
        write!(output, "<{}><template shadowrootmode=\"open\">", tag.0)?;
        match template {
            None => Ok(Left(output)),
            Some(template) => {
                match render_node_instance(world, &template)? {
                    Left(partial) => {
                        output.write_str(&partial)?;
                        return Ok(Left(output))
                    }
                    Right(complete) => {
                        output.push_str(&complete);
                    }
                };
                write!(output, "</template>")?;
                if let Some(child_nodes) = children {
                    for child in child_nodes.0.iter() {
                        match render_node_instance(world, child)? {
                            Left(partial) => {
                                output.write_str(&partial)?;
                                return Ok(Left(output))
                            }
                            Right(complete) => {
                                output.push_str(&complete);
                            }
                        };
                    }
                }
                write!(output, "</{}>", tag.0)?;
                Ok(Right(output))
            }
        }
    }


    ///Render a node instance, will be partial if component templates don't exist yet
    pub fn render_node_instance(world: &World, node: &Node) -> Result<Either<String, String>, std::fmt::Error> {
        match node {
            Node::Text(string) => Ok(Right(string.to_owned())),
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
                        match render_node_instance(world, child)? {
                            Left(partial) => {
                                output.write_str(&partial)?;
                                return Ok(Left(output))
                            }
                            Right(complete) => {
                                output.push_str(&complete);
                            }
                        };
                    }
                    write!(output, "</{}>", tag)?;
                }
                Ok(Right(output))
            }
            Node::Component(entity) =>
                render_component_instance(world, *entity),

            //We place comments around our templated expression so that we can locate it for hydration
            Node::Expression(id, exp_fn) => Ok(Right(format!("<!--#exp:{id}-->{}<!--/exp:{id}-->", exp_fn()))),
        }
    }
  }
}

// /// Construct the view and put it into the ecs
// pub async fn construct_entity_view(
//     entity: Entity,
//     serial_server_data: Option<SerialServerData>,
// ) -> Result<(), JsValue> {
//     let has_node: bool;
//     let view: AnyView;
//     {
//         let mut world = WORLD.write().unwrap();
//         {
//             let (selector, has_server_data, i_has_node, i_view) = world
//                 .query::<(&Selector, Has<ServerData>, Has<NodeRef>, &AnyView)>()
//                 .get(&world, entity)
//                 .unwrap();
//             has_node = i_has_node;
//             view = i_view.clone();
//             if !has_server_data {
//                 if let Some(server_data) = serial_server_data.as_ref().and_then(|s| s.get(selector))
//                 {
//                     let mut entity_ref = world.entity_mut(entity);
//                     entity_ref.insert(server_data);
//                 }
//             }
//         }
//     }
//     if !has_node {
//         let node_ref = view.build().await;
//         {
//             println!("Building view for entity {:?} {:?}", entity, node_ref);
//             let mut world = WORLD.write().unwrap();
//             let mut entity_ref = world.entity_mut(entity);
//             entity_ref.insert(node_ref.clone());
//         }
//         construct_entity_view_with_node(entity, node_ref.into(), serial_server_data).await?;
//     }
//     Ok(())
// }

// #[async_recursion::async_recursion]
// async fn construct_entity_view_with_node(
//     entity: Entity,
//     node_ref: NodeRef,
//     serial_server_data: Option<SerialServerData>,
// ) -> Result<(), JsValue> {
//     match node_ref.as_ref() {
//         Node::Component {
//             entity,
//             child_nodes,
//         } => {
//             //Construct view for the entity, if necessary (it may have been constructed in a previous visit)
//             construct_entity_view(*entity, serial_server_data.clone()).await?;
//             for child in child_nodes.into_iter() {
//                 construct_entity_view_with_node(
//                     *entity,
//                     child.to_owned(),
//                     serial_server_data.clone(),
//                 )
//                 .await?;
//             }
//             Ok(())
//         }
//         Node::HtmlElement {
//             element: _,
//             child_nodes,
//         } => {
//             for child in child_nodes.into_iter() {
//                 construct_entity_view_with_node(
//                     entity,
//                     child.to_owned(),
//                     serial_server_data.clone(),
//                 )
//                 .await?;
//             }
//             Ok(())
//         }
//         #[cfg(target_arch = "wasm32")]
//         Node::Expression(id, expr) => {
//             use web_sys::window;
//             let document = window()
//                 .expect("No window")
//                 .document()
//                 .expect("no document");
//             web_sys::console::log_1(&format!("Binding expression {:?}", id).into());
//             let host_component: web_sys::Element;
//             {
//                 let world = WORLD.read().unwrap();
//                 let entity_ref = world.entity(entity);
//                 let selector = entity_ref
//                     .get::<Selector>()
//                     .expect("Entity needs a selector");
//                 host_component = document
//                     .query_selector(&selector.to_string())?
//                     .expect("id should exist");
//             }
//             // We walk the host component to query its child nodes
//             let comments_walker =
//                 document.create_tree_walker_with_what_to_show(&host_component, SHOW_COMMENT)?;
//             replace_expression_comments(&comments_walker, &id, &expr)?;
//             // And again for its shadow root template
//             if let Some(shadow_root) = host_component.shadow_root() {
//                 let comments_walker = document.create_tree_walker_with_what_to_show(
//                     &shadow_root.get_root_node(),
//                     SHOW_COMMENT,
//                 )?;
//                 replace_expression_comments(&comments_walker, &id, &expr)?;
//             }
//             Ok(())
//         }
//         _ => Ok(()),
//     }
// }

// /// Find start and end expression comments for given expression id, empty out the inbetween and insert expression result
// fn replace_expression_comments(
//     comments_walker: &TreeWalker,
//     id: &str,
//     expr: &dyn Fn() -> String,
// ) -> Result<(), JsValue> {
//     loop {
//         web_sys::console::log_1(&format!("Iding comment {:?}", id).into());
//         let start_comment = comments_walker.next_node()?;
//         match start_comment {
//             Some(start_comment) => {
//                 let start_comment = start_comment.dyn_into::<Comment>()?;
//                 if start_comment
//                     .data()
//                     .starts_with(format!("#exp:{id}").as_str())
//                 {
//                     loop {
//                         let end_comment = comments_walker.next_sibling()?;
//                         match end_comment {
//                             Some(end_comment) => {
//                                 let end_comment = end_comment.dyn_into::<Comment>()?;
//                                 if end_comment
//                                     .data()
//                                     .starts_with(format!("/exp:{id}").as_str())
//                                 {
//                                     return replace_between_comments(
//                                         start_comment,
//                                         end_comment,
//                                         expr,
//                                     );
//                                 }
//                             }
//                             None => break,
//                         }
//                     }
//                 }
//             }
//             None => {
//                 web_sys::console::log_1(&format!("Out of comments! {:?}", id).into());
//                 break;
//             }
//         }
//     }
//     Ok(())
// }

// /// Replace all nodes between two comments with a text node containing expression results
// fn replace_between_comments(
//     start_comment: Comment,
//     end_comment: Comment,
//     expr: &dyn Fn() -> String,
// ) -> Result<(), JsValue> {
//     let comment_parent = start_comment
//         .parent_node()
//         .expect("No parent node on comment");
//     let mut n = start_comment
//         .next_sibling()
//         .expect("Nodes after start comment should have more sibilings");
//     while n != **end_comment {
//         let to_remove = n.to_owned();
//         n = n
//             .next_sibling()
//             .expect("Nodes after start comment should have more sibilings");
//         comment_parent.remove_child(&to_remove)?;
//     }
//     start_comment.after_with_str_1(format!("JS: {}", expr().as_str()).as_str())?;
//     Ok(())
// }
