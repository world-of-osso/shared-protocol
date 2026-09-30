//! Protocol types of one build.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Component, Serialize, Deserialize, PartialEq)]
pub struct LayoutComponent {
    pub current: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct LayoutMessage {
    pub id: u32,
}
