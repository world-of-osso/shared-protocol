//! A unit's body for line of sight: its feet and eye point, and the volume a witness ray
//! ends in (game-server docs/specs/line-of-sight.md, Eye point and Player casts).

use bevy::math::Vec3;

/// Radius of the body volume around the feet-to-eyes segment: TrinityCore
/// `DEFAULT_PLAYER_BOUNDING_RADIUS` / `DEFAULT_WORLD_OBJECT_SIZE` (ObjectDefines.h).
pub const BODY_RADIUS: f32 = 0.388;

/// Where a unit stands and sees from, in Bevy space.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Body {
    pub feet: Vec3,
    pub eyes: Vec3,
}

impl Body {
    /// A unit standing at `feet`, facing `yaw` (Bevy `Rotation.y`: forward is
    /// `(sin yaw, 0, cos yaw)`), whose eye table point is `eye` (model space, WoW axes: X
    /// forward, Y left, Z up).
    pub fn placed(feet: Vec3, yaw: f32, [forward, left, up]: [f32; 3]) -> Self {
        let (sin, cos) = yaw.sin_cos();
        let offset = Vec3::new(sin, 0.0, cos) * forward + Vec3::new(cos, 0.0, -sin) * left;
        Self {
            feet,
            eyes: feet + offset + Vec3::Y * up,
        }
    }

    /// Halfway between the feet and the eyes.
    pub fn chest(&self) -> Vec3 {
        (self.feet + self.eyes) * 0.5
    }

    /// The points a line of sight search aims at, in order: eyes, chest, feet.
    pub fn aim_points(&self) -> [Vec3; 3] {
        [self.eyes, self.chest(), self.feet]
    }

    /// Whether `point` is inside the body volume, grown by `tolerance`: the capsule of
    /// radius [`BODY_RADIUS`] around the feet-to-eyes segment.
    pub fn contains(&self, point: Vec3, tolerance: f32) -> bool {
        let axis = self.eyes - self.feet;
        let along = (point - self.feet).dot(axis) / axis.length_squared().max(f32::EPSILON);
        let nearest = self.feet + axis * along.clamp(0.0, 1.0);
        point.distance(nearest) <= BODY_RADIUS + tolerance
    }
}

#[cfg(test)]
#[path = "body_tests.rs"]
mod tests;
