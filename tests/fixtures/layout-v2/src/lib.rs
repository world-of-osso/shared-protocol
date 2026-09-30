//! The same protocol types as `layout-v1`, each with one field added.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Component, Serialize, Deserialize, PartialEq)]
pub struct LayoutComponent {
    pub current: u32,
    pub max: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct LayoutMessage {
    pub id: u32,
    pub flags: u8,
}
