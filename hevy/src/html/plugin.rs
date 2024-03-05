use std::{
    collections::HashMap,
    rc::Rc,
    sync::{Arc, RwLock},
};
use tokio::sync::mpsc;

use crate::{
    html::{
        attributes::{add_attributes_to_render_attributes, reset_render_attributes},
        render::{add_render_tags, add_render_tags_for_text, render_tags_to_output, RenderOutput},
        styles::apply_styles,
    },
    r#async::{AsyncContext, AsyncReceivers, AsyncTasks},
};

use super::tag::{Main, Time, *};
use bevy::prelude::*;
use either::Either;

#[derive(Debug, Hash, Eq, PartialEq, Clone, SystemSet)]
enum HtmlRenderSet {
    ApplyTags,
    ApplyAttributes,
    AddTags,
    RenderTags,
}

macro_rules! add_tag_systems {
    ($app:ident, ($($tag:ident),+), $($group:tt),+) => {
      add_tag_systems!($app, ($($tag),*));
      add_tag_systems!($app, $($group),*);
    };
    ($app:ident, ($($tag:ident),+)) => {
      $app.add_systems(Update, ($(make_tag::<$tag>),*).in_set(HtmlRenderSet::ApplyTags));
    };
}

pub struct RenderHtmlPlugin;
impl Plugin for RenderHtmlPlugin {
    fn build(&self, app: &mut App) {
        add_tag_systems!(
            app,
            (Div, H1, H2, H3, H4, H5, H6, P, Span, A),
            (Img, Button, Input, Label, Select, Option, Textarea, Form, Table, Tr),
            (Td, Th, Thead, Tbody, Tfoot, Ul, Ol, Li, Dl, Dt),
            (Dd, Section, Header, Footer, Main, Article, Aside, Nav, Address, Blockquote),
            (Details, Summary, Dialog, Menu, MenuItem, Figure, Figcaption, Audio, Video),
            (
                Canvas, Embed, Object, Source, Track, Map, Area, Math, Svg, Iframe, Frame,
                Frameset, Noframes
            ),
            (
                B, Strong, I, Em, Mark, Small, Del, Ins, Sub, Sup, Pre, Code, Var, Samp, Kbd, Q,
                Cite, Abbr
            ),
            (Dfn, Time, Progress, Meter, Br, Wbr, Template, Slot, Script, Noscript, Style, Meta),
            (Link, Title, Base, Head, Html, Body)
        );
        // Reset our render attribute components
        app.add_systems(
            Update,
            reset_render_attributes.after(HtmlRenderSet::ApplyTags),
        );

        // Apply attributes to render attributes
        app.add_systems(
            PostUpdate,
            (add_attributes_to_render_attributes).in_set(HtmlRenderSet::ApplyAttributes),
        );

        // Apply styles to render attributes
        app.add_systems(
            PostUpdate,
            (apply_styles).in_set(HtmlRenderSet::ApplyAttributes),
        );

        // Render out our tags to render tags
        app.add_systems(
            PostUpdate,
            (add_render_tags, add_render_tags_for_text)
                .in_set(HtmlRenderSet::AddTags)
                .after(HtmlRenderSet::ApplyAttributes),
        );

        // Walk render tags from page down and write to output
        app.add_systems(
            PostUpdate,
            render_tags_to_output
                .in_set(HtmlRenderSet::RenderTags)
                .after(HtmlRenderSet::AddTags)
                .after(HtmlRenderSet::ApplyAttributes),
        );
        let (world_callback_tx, world_callback_rx) = mpsc::channel(100);
        let (commands_callback_tx, commands_callback_rx) = mpsc::channel(100);
        app.insert_resource(AsyncTasks {
            map: HashMap::new(),
            world_callback_tx,
            commands_callback_tx,
        });
        app.insert_resource(RenderOutput(Either::Left(String::new())));
        app.insert_resource(AsyncReceivers {
            world_callback_rx: Arc::new(RwLock::new(world_callback_rx)),
            commands_callback_rx: Arc::new(RwLock::new(commands_callback_rx)),
        });
    }
}
