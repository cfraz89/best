// mod my_h1;

use bevy::prelude::*;
use elementary::html::*;
use elementary::prelude::*;

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
        <Div StyleAttr(hash_map! {"color" => "red"})> {
            "Yolo"
        }
    });
}
