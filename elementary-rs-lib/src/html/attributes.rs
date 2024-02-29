use std::collections::HashMap;

use bevy::prelude::*;

#[derive(Component, Debug)]
pub struct Attributes(pub HashMap<String, String>);
