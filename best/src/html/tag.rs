use bevy::prelude::*;

macro_rules! make_tag {
    ($name:ident, $tag:literal) => {
        #[derive(Component, Debug, Clone, Copy)]
        pub struct $name;

        impl Into<Tag> for $name {
            fn into(self) -> Tag {
                Tag($tag)
            }
        }
    };
}

#[derive(Component, Clone)]
pub struct Tag(pub &'static str);

/// All the shorthand components use this system to add the Tag component
pub fn make_tag<T: Into<Tag> + Component + Copy>(
    mut commands: Commands,
    query: Query<(Entity, &T), Without<Tag>>,
) {
    for (entity, t) in query.iter() {
        let tag: Tag = (*t).into();
        commands.entity(entity).insert(tag);
    }
}

make_tag!(Div, "div");
make_tag!(H1, "h1");
make_tag!(H2, "h2");
make_tag!(H3, "h3");
make_tag!(H4, "h4");
make_tag!(H5, "h5");
make_tag!(H6, "h6");
make_tag!(P, "p");
make_tag!(Span, "span");
make_tag!(A, "a");
make_tag!(Img, "img");
make_tag!(Button, "button");
make_tag!(Input, "input");
make_tag!(Label, "label");
make_tag!(Select, "select");
make_tag!(Option, "option");
make_tag!(Textarea, "textarea");
make_tag!(Form, "form");
make_tag!(Table, "table");
make_tag!(Tr, "tr");
make_tag!(Td, "td");
make_tag!(Th, "th");
make_tag!(Thead, "thead");
make_tag!(Tbody, "tbody");
make_tag!(Tfoot, "tfoot");
make_tag!(Ul, "ul");
make_tag!(Ol, "ol");
make_tag!(Li, "li");
make_tag!(Dl, "dl");
make_tag!(Dt, "dt");
make_tag!(Dd, "dd");
make_tag!(Section, "section");
make_tag!(Header, "header");
make_tag!(Footer, "footer");
make_tag!(Main, "main");
make_tag!(Article, "article");
make_tag!(Aside, "aside");
make_tag!(Nav, "nav");
make_tag!(Address, "address");
make_tag!(Blockquote, "blockquote");
make_tag!(Details, "details");
make_tag!(Summary, "summary");
make_tag!(Dialog, "dialog");
make_tag!(Menu, "menu");
make_tag!(MenuItem, "menuitem");
make_tag!(Figure, "figure");
make_tag!(Figcaption, "figcaption");
make_tag!(Audio, "audio");
make_tag!(Video, "video");
make_tag!(Canvas, "canvas");
make_tag!(Embed, "embed");
make_tag!(Object, "object");
make_tag!(Source, "source");
make_tag!(Track, "track");
make_tag!(Map, "map");
make_tag!(Area, "area");
make_tag!(Math, "math");
make_tag!(Svg, "svg");
make_tag!(Iframe, "iframe");
make_tag!(Frame, "frame");
make_tag!(Frameset, "frameset");
make_tag!(Noframes, "noframes");
make_tag!(B, "b");
make_tag!(Strong, "strong");
make_tag!(I, "i");
make_tag!(Em, "em");
make_tag!(Mark, "mark");
make_tag!(Small, "small");
make_tag!(Del, "del");
make_tag!(Ins, "ins");
make_tag!(Sub, "sub");
make_tag!(Sup, "sup");
make_tag!(Pre, "pre");
make_tag!(Code, "code");
make_tag!(Var, "var");
make_tag!(Samp, "samp");
make_tag!(Kbd, "kbd");
make_tag!(Q, "q");
make_tag!(Cite, "cite");
make_tag!(Abbr, "abbr");
make_tag!(Dfn, "dfn");
make_tag!(Time, "time");
make_tag!(Progress, "progress");
make_tag!(Meter, "meter");
make_tag!(Br, "br");
make_tag!(Wbr, "wbr");
make_tag!(Template, "template");
make_tag!(Slot, "slot");
make_tag!(Script, "script");
make_tag!(Noscript, "noscript");
make_tag!(Style, "style");
make_tag!(Meta, "meta");
make_tag!(Link, "link");
make_tag!(Title, "title");
make_tag!(Base, "base");
make_tag!(Head, "head");
make_tag!(Html, "html");
make_tag!(Body, "body");
