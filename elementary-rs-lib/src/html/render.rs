use either::Either;

use crate::text::Text;

use super::{async_wait::AsyncWait, attributes::RenderAttributes, tag::Tag};
use bevy::prelude::*;
use std::fmt::Write;

#[derive(Component, Clone)]
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

#[derive(Component)]
pub struct Page;

static NO_SELF_CLOSE_TAGS: [&str; 3] = ["script", "style", "template"];

///Render a node instance, will be partial if component templates don't exist yet
pub(crate) fn render_tag(
    tag: &Tag,
    attributes: &RenderAttributes,
    children: Option<&Children>,
    async_wait: Option<&AsyncWait>,
) -> Result<RenderTag, std::fmt::Error> {
    if async_wait.is_some_and(|s| s.0 > 0) {
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
//Without<RenderTag> is important to ensure we don't reset tags that are already rendered/streamed
pub(crate) fn add_render_tags(
    mut commands: Commands,
    query: Query<
        (
            Entity,
            &Tag,
            &RenderAttributes,
            Option<&Children>,
            Option<&AsyncWait>,
        ),
        Without<RenderTag>,
    >,
) {
    for (entity, tag, attributes, children, async_wait) in &query {
        commands.entity(entity).insert(
            render_tag(tag, attributes, children, async_wait).expect("Error rendering tag"),
        );
    }
}

//System to actually add render tags to text components.
//Without<RenderTag> is important to ensure we don't reset render status of text that has been streamed
pub(crate) fn add_render_tags_for_text(
    mut commands: Commands,
    query: Query<(Entity, &Text), Without<RenderTag>>,
) {
    for (entity, text) in &query {
        commands
            .entity(entity)
            .insert(RenderTag::Text(text.0.clone()));

        dbg!("Added render tag to entity", entity);
    }
}

//Recursively render an entity, including its children
pub(crate) fn render_entity_tags(
    world: &mut World,
    entity: Entity,
) -> Result<Either<String, String>, std::fmt::Error> {
    dbg!("Looking for render tag on", entity);
    let render_tag: RenderTag = { world.get::<RenderTag>(entity).unwrap().clone() };
    match render_tag {
        RenderTag::Consumed => Ok(Either::Right("".to_string())),
        RenderTag::OpenConsumed { close } => Ok(Either::Right(close.to_string())),
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
            let children: Option<Vec<Entity>> = {
                world
                    .get::<Children>(entity)
                    .map(|c| c.iter().map(|e| e.to_owned()).collect::<Vec<_>>())
            };
            if let Some(children) = children {
                world.entity_mut(entity).insert(RenderTag::OpenConsumed {
                    close: close.to_string(),
                });
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
            world.entity_mut(entity).insert(RenderTag::Consumed);
            write!(output, "{}", close)?;
            Ok(Either::Right(output.to_string()))
        }
    }
}

//System to consume our tags into output resource
pub(crate) fn render_tags_to_output(world: &mut World) {
    let entity = world.query_filtered::<Entity, With<Page>>().single(world);
    let rendered = render_entity_tags(world, entity).unwrap();
    let mut output = world.get_resource_mut::<RenderOutput>().unwrap();
    output.0 = rendered;
}
