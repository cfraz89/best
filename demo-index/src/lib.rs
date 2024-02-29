// mod my_h1;

use std::collections::HashMap;

use bevy::prelude::*;
use elementary_rs_lib::html::{
    render::Page,
    style::Style,
    tag::{Div, H1},
};
use elementary_rs_lib::logic::{Else, If};
use elementary_rs_lib::text::Text;
use elementary_rs_macros::ecn;

pub fn init_page(mut commands: Commands) {
    ecn!(commands,
    <Div Page> {
        <If(true)> {
            <H1 Text("Hello")> {
                <Text("World")>
            }
            <Div> {
                <Text("Hello")>
            }
        }
        <Else> {
            "Blah"
        }
        <SomeComponent>
    });
}
