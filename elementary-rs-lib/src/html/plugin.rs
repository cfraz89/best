use super::tag::{Main, Time, *};
use bevy::prelude::*;

struct RenderHtmlPlugin {}

#[derive(Debug, Hash, Eq, PartialEq, Clone, SystemSet)]
enum HtmlRenderSet {
    ApplyTags,
    ApplyAttributes,
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
    }
}
