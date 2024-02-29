// mod my_h1;

use bevy::prelude::*;
use elementary_rs_lib::html::{
    render::Page,
    tag::{Div, H1},
};
use elementary_rs_lib::text::Text;

pub fn init_page(commands: Commands) {
    make_page_entities(commands);
}

pub fn make_page_entities(mut commands: Commands) -> Entity {
    // entity!(world,
    // <Div> {
    //     <If(true)> {
    //         <MyH1 Title("Hello")> {
    //             <Text("World")>
    //         }
    //         <Div> {
    //             <Text("Hello")>
    //         }
    //     }
    //     <Else> {
    //         Text("Blah")
    //     }
    //     <SomeComponent>
    // })
    commands
        .spawn((Page, Div))
        .with_children(|builder| {
            builder.spawn(H1).with_children(|builder| {
                builder.spawn(Text("World".to_string()));
            });
            builder.spawn(Div).with_children(|builder| {
                builder.spawn(Text("Hello".to_string()));
            });
        })
        .id()
}
