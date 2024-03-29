use either::Either;

use crate::r#async::AsyncTasks;

use super::{attributes::RenderAttributes, tag::Tag, text::Text};
use bevy::prelude::*;
use std::fmt::Write;

#[derive(Component, Clone, Debug)]
pub(crate) enum RenderTag {
    Waiting,
    SelfClosing(String),
    OpenClose { open: String, close: String },
    Consumed,
    OpenConsumed { close: String },
    Text(String),
}

#[derive(Resource)]
pub(crate) struct RenderOutput(pub(crate) Either<String, String>);

static NO_SELF_CLOSE_TAGS: [&str; 3] = ["script", "style", "template"];

///Render a node instance, will be partial if component templates don't exist yet
pub(crate) fn get_render_tag(
    entity: Entity,
    tag: &Tag,
    attributes: &RenderAttributes,
    children: Option<&Children>,
    async_tasks: &Res<AsyncTasks>,
) -> Result<RenderTag, std::fmt::Error> {
    if async_tasks.map.get(&entity).is_some_and(|s| s.len() > 0) {
        return Ok(RenderTag::Waiting);
    }
    let mut open = String::new();
    let attributes_string = attributes
        .0
        .iter()
        .map(|(k, v)| format!(" {}=\"{}\"", k, v))
        .collect::<Vec<String>>()
        .join(" ");
    write!(open, "<{}{}", tag.0, attributes_string)?;
    // Some tags don't like self_closing
    if !children.is_some_and(|c| c.len() > 0) && !NO_SELF_CLOSE_TAGS.contains(&tag.0) {
        write!(open, " />")?;
        return Ok(RenderTag::SelfClosing(open));
    } else {
        write!(open, ">")?;
    }
    Ok(RenderTag::OpenClose {
        open,
        close: format!("</{}>", tag.0),
    })
}

//System to actually add the tags
pub(crate) fn add_render_tags(
    mut commands: Commands,
    query: Query<(
        Entity,
        &Tag,
        &RenderAttributes,
        Option<&RenderTag>,
        Option<&Children>,
    )>,
    async_tasks: Res<AsyncTasks>,
) {
    for (entity, tag, attributes, render_tag, children) in &query {
        match render_tag {
            None | Some(RenderTag::Waiting) => {
                dbg!("Adding render tag to entity", entity);
                commands.entity(entity).insert(
                    get_render_tag(entity, tag, attributes, children, &async_tasks)
                        .expect("Error rendering tag"),
                );
            }
            _ => continue,
        }
    }
}

//System to actually add render tags to text components.
//Without<RenderTag> is important to ensure we don't reset render status of text that has been streamed
pub(crate) fn add_render_tags_for_text(
    mut commands: Commands,
    query: Query<(Entity, &Text, Option<&RenderTag>)>,
) {
    for (entity, text, render_tag) in &query {
        match render_tag {
            None | Some(RenderTag::Waiting) => {
                commands
                    .entity(entity)
                    .insert(RenderTag::Text(text.0.to_string()));
            }
            _ => continue,
        }
    }
}

//Recursively render an entity, including its children
pub(crate) fn render_entity_tags(
    world: &mut World,
    entity: Entity,
) -> Result<Either<String, String>, std::fmt::Error> {
    let render_tag: RenderTag = {
        world
            .get::<RenderTag>(entity)
            .expect("Entity should have a render tag by now")
            .clone()
    };
    dbg!(entity, render_tag.clone());
    match render_tag {
        RenderTag::Consumed => Ok(Either::Right("".to_string())),
        RenderTag::OpenConsumed { close } => match render_children(entity, world)? {
            Either::Left(partial) => Ok(Either::Left(partial)),
            Either::Right(s) => {
                let mut output = s;
                world.entity_mut(entity).insert(RenderTag::Consumed);
                write!(output, "{}", close)?;
                Ok(Either::Right(output))
            }
        },
        RenderTag::Waiting => Ok(Either::Left("".to_string())),
        RenderTag::SelfClosing(s) => {
            world.entity_mut(entity).insert(RenderTag::Consumed);
            Ok(Either::Right(s.to_string()))
        }
        RenderTag::Text(s) => {
            world.entity_mut(entity).insert(RenderTag::Consumed);
            Ok(Either::Right(s.to_string()))
        }
        RenderTag::OpenClose { open, close } => {
            let mut output = open;
            match render_children(entity, world)? {
                Either::Left(partial) => {
                    output.write_str(&partial)?;
                    world
                        .entity_mut(entity)
                        .insert(RenderTag::OpenConsumed { close });
                    Ok(Either::Left(output))
                }
                Either::Right(s) => {
                    output.push_str(&s);
                    world.entity_mut(entity).insert(RenderTag::Consumed);
                    write!(output, "{}", close)?;
                    Ok(Either::Right(output))
                }
            }
        }
    }
}

fn render_children(
    entity: Entity,
    world: &mut World,
) -> Result<Either<String, String>, std::fmt::Error> {
    let mut output = String::new();
    let children: Option<Vec<Entity>> = {
        world
            .get::<Children>(entity)
            .map(|c| c.iter().map(|e| e.to_owned()).collect::<Vec<_>>())
    };
    if let Some(children) = children {
        for child in children.into_iter() {
            match render_entity_tags(world, child)? {
                Either::Left(partial) => {
                    output.write_str(&partial)?;
                    return Ok(Either::Left(output.to_string()));
                }
                Either::Right(s) => output.push_str(&s),
            }
        }
    }
    Ok(Either::Right(output))
}

/// System to consume our tags into output resource
/// Tries to find the root element via a query filter, and renders that
pub(crate) fn render_tags_to_output(world: &mut World) {
    let entity = world
        .query_filtered::<Entity, (Or<(With<Tag>, With<Text>)>, Without<Parent>)>()
        .single(world);
    let rendered = render_entity_tags(world, entity).unwrap();
    let mut output = world.get_resource_mut::<RenderOutput>().unwrap();
    output.0 = rendered;
}
