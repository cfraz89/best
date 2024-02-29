// mod my_h1;

use bevy::prelude::*;
use elementary_rs_lib::html::{
    render::Page,
    style::Style,
    tag::{Div, H1},
};
use elementary_rs_lib::prelude::*;
use elementary_rs_macros::ecn;

pub fn init_page(mut commands: Commands) {
    // ecn!(commands,
    // <Div> {
    //     <If(true)> {
    //         <H1> {
    //             <Text("World")>
    //         }
    //         <Div> {
    //             <Text("Hello")>
    //         }
    //     }
    //     <Else> {
    //         "Blah"
    //     }
    //     <SomeComponent>
    // });
    ecn!(commands,
    <Div Page> {
        "Hello"
        <Div Style(hash_map! {"color" => "red"})> {
            "Yolo"
        }
    });
}
