//! The surface a character stands on, one rule for client prediction and
//! server authority (game-engine docs/specs/wmo-floor-collision.md).
//!
//! Candidates are the ADT terrain height and the walkable WMO collision faces
//! under the feet. The ground is the highest candidate no higher than the feet
//! plus [`STEP_UP_HEIGHT`]; a floor above that reach is a ceiling or a roof.
//! Positions and heights are in Bevy world space (Y up).

use bevy::math::{Affine3A, EulerRot, Quat, Vec2, Vec3, Vec3Swizzles};

use crate::movement::MAX_SLOPE_ANGLE;
use crate::navmesh::TILE_SIZE;

mod wmo;

pub use wmo::WmoGroupCollision;

/// Highest surface above the feet a character steps onto (WoWee grounded
/// step budget, `camera_controller.cpp` `stepUpBudget = 1.6f`).
pub const STEP_UP_HEIGHT: f32 = 1.6;

/// How far below the feet WMO floors are searched (AzerothCore
/// `DEFAULT_HEIGHT_SEARCH`).
pub const FLOOR_SEARCH_DEPTH: f32 = 50.0;

/// Which kind of surface supports the character.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Surface {
    Terrain,
    Wmo,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Ground {
    pub height: f32,
    pub surface: Surface,
}

/// A placed WMO: its collision groups and placement transform.
pub struct WmoCollision {
    world_from_local: Affine3A,
    local_from_world: Affine3A,
    /// World XZ extent of the group bounding boxes.
    world_min: Vec2,
    world_max: Vec2,
    groups: Vec<std::sync::Arc<WmoGroupCollision>>,
}

impl WmoCollision {
    /// `world_from_local` maps WMO-local Bevy space (`[x, z, -y]` of the file's
    /// coordinates) to the world, as the WMO root's transform does.
    pub fn new(world_from_local: Affine3A, groups: Vec<std::sync::Arc<WmoGroupCollision>>) -> Self {
        let (world_min, world_max) = groups
            .iter()
            .flat_map(|group| group.local_corners())
            .map(|corner| world_from_local.transform_point3(corner).xz())
            .fold(
                (Vec2::splat(f32::MAX), Vec2::splat(f32::MIN)),
                |(min, max), corner| (min.min(corner), max.max(corner)),
            );
        Self {
            world_from_local,
            local_from_world: world_from_local.inverse(),
            world_min,
            world_max,
            groups,
        }
    }

    /// World heights of the walkable faces crossed by the vertical segment at
    /// world `(x, z)` from `top_y` down to `bottom_y`.
    pub fn floor_heights(&self, x: f32, z: f32, top_y: f32, bottom_y: f32, out: &mut Vec<f32>) {
        let column = Vec2::new(x, z);
        if column.cmplt(self.world_min).any() || column.cmpgt(self.world_max).any() {
            return;
        }
        let top = self
            .local_from_world
            .transform_point3(Vec3::new(x, top_y, z));
        let bottom = self
            .local_from_world
            .transform_point3(Vec3::new(x, bottom_y, z));
        for group in &self.groups {
            group.walkable_hits(top, bottom, &self.world_from_local, out);
        }
    }
}

/// The ground under `feet`: `terrain` is the terrain height at the feet's
/// column (`None` off loaded terrain), `wmos` the WMOs placed around it.
pub fn ground_at<'a>(
    feet: Vec3,
    terrain: Option<f32>,
    wmos: impl IntoIterator<Item = &'a WmoCollision>,
) -> Option<Ground> {
    let mut floors = Vec::new();
    let top = feet.y + STEP_UP_HEIGHT;
    let bottom = feet.y - FLOOR_SEARCH_DEPTH;
    for wmo in wmos {
        wmo.floor_heights(feet.x, feet.z, top, bottom, &mut floors);
    }
    select_ground(feet.y, terrain, floors)
}

/// Highest candidate within step reach of `feet_y`; a WMO floor wins a tie
/// with the terrain it paves over.
pub fn select_ground(
    feet_y: f32,
    terrain: Option<f32>,
    wmo_floors: impl IntoIterator<Item = f32>,
) -> Option<Ground> {
    let reach = feet_y + STEP_UP_HEIGHT;
    let terrain = terrain.map(|height| Ground {
        height,
        surface: Surface::Terrain,
    });
    let floors = wmo_floors.into_iter().map(|height| Ground {
        height,
        surface: Surface::Wmo,
    });
    terrain
        .into_iter()
        .chain(floors)
        .filter(|ground| ground.height <= reach)
        .reduce(|best, next| {
            if next.height >= best.height {
                next
            } else {
                best
            }
        })
}

/// Smallest world-up normal component of a walkable WMO face.
pub fn min_walkable_normal_y() -> f32 {
    MAX_SLOPE_ANGLE.cos()
}

/// Bevy world position of an ADT MODF/MDDF placement position.
pub fn placement_position(raw: [f32; 3]) -> Vec3 {
    let center = 32.0 * TILE_SIZE;
    Vec3::new(center - raw[2], raw[1], raw[0] - center)
}

/// Bevy world position of a WDT global WMO placement (MPHD flag 0x1, a map made of
/// one WMO). Unlike an ADT placement it is not offset from the map corner: Stormwind
/// Stockade's WDT 791060 places WMO 108631 at raw (0, 0, 0), and the Stockade entrance
/// (world_safe_locs 3599, WoW (56.68, 0.62, -19.27)) lies inside that WMO's MOHD bounds.
pub fn global_wmo_placement_position(raw: [f32; 3]) -> Vec3 {
    Vec3::new(-raw[2], raw[1], raw[0])
}

/// Bevy rotation of an ADT MODF/MDDF placement rotation (degrees): stored
/// `[X, Y, Z]` becomes model rotation `[Z, Y - 180, -X]`, applied in YZX order.
pub fn placement_rotation(rot: [f32; 3]) -> Quat {
    let bank_x = rot[2].to_radians();
    let heading_y = (rot[1] - 180.0).to_radians();
    let attitude_z = (-rot[0]).to_radians();
    Quat::from_euler(EulerRot::YZX, heading_y, attitude_z, bank_x)
}

#[cfg(test)]
#[path = "ground_tests.rs"]
mod tests;
