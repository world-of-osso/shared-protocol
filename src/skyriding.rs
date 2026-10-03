//! Skyriding (Retail "dragonriding", Flight Style: Skyriding 404464): momentum flight the
//! client integrates and the server bounds by energy.
//!
//! Retail sends the parameters (`SMSG_MOVE_SET_ADV_FLYING_*`, TrinityCore a352b1fa
//! `Unit::UpdateAdvFlyingSpeed`, Unit.cpp:9042-9160) from the `FlightCapability` row of the
//! rider's `MountCapability` (`Unit::SetFlightCapabilityID`, Unit.cpp:9020); the client's
//! integrator itself is unpublished. The model here is ours, driven by those parameters:
//! - a launch is ballistic (gravity and air friction, no lift) until it falls with at
//!   least `glide_start_min_height` under it, then the mount glides (Icy Veins Skyriding
//!   Guide: "While falling from a high location, your mount extends its wings");
//! - gliding, lift turns the velocity toward the mount's facing and pitch at
//!   `lift_coefficient` × speed radians a second (lift ∝ speed², so a slow mount stalls),
//!   without changing the speed;
//! - gravity acts on the velocity, so diving turns height into speed and climbing bleeds it;
//! - air friction takes `air_friction` yards a second off the speed every second;
//! - gravity never pushes the speed past `max_vel`.
//!
//! Lift and friction never add energy, so a skyrider's speed after falling `drop` yards is
//! at most `sqrt(v0² + 2·GRAVITY·drop)` (`speed_after`), the server's bound.

use bevy::math::Vec3;

/// `Movement::gravity` (TrinityCore MovementTypedefs.h:81), yards/s².
pub const GRAVITY: f32 = 19.291_103;

/// Upward speed of the part-1 takeoff, yards/s: Skyward Ascent 372610 effect 0 base points
/// 450, read as 450% of `BASE_MOVEMENT_SPEED` 7 (PaperDollFrame.lua:38). The reading is an
/// assumption; Skyward Ascent itself (vigor, charges) is skyriding part 2.
pub const LAUNCH_SPEED: f32 = 4.5 * 7.0;

/// The `FlightCapability` fields the model reads.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FlightCapability {
    /// `AirFriction`: speed lost per second, yards/s².
    pub air_friction: f32,
    /// `MaxVel`: the most gravity accelerates the mount to, yards/s.
    pub max_vel: f32,
    /// `LiftCoefficient`: lift turn rate per yard/s of speed, radians/s.
    pub lift_coefficient: f32,
    /// `GlideStartMinHeight`: the height over the ground a fall must have to glide, yards.
    pub glide_start_min_height: f32,
    /// `PitchingRateUpMin` / `PitchingRateDownMin`, radians/s.
    pub pitching_rate_up: f32,
    pub pitching_rate_down: f32,
}

/// `FlightCapability` 11, the capability of Skyriding `MountCapability` 494 (Retail DB2
/// 12.1.0.69933, FileDataID 4501047, layout 5CF8CAA8, from the local CASC install).
pub const SKYRIDING: FlightCapability = FlightCapability {
    air_friction: 1.5,
    max_vel: 65.0,
    lift_coefficient: 0.07,
    glide_start_min_height: 7.5,
    pitching_rate_up: 180.0_f32.to_radians(),
    pitching_rate_down: 180.0_f32.to_radians(),
};

/// A skyrider in the air.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Glider {
    pub velocity: Vec3,
    /// The mount's pitch, radians (+ up).
    pub pitch: f32,
    /// Wings out: lift steers the velocity.
    pub gliding: bool,
}

/// The direction a mount facing `yaw` (forward `(sin yaw, 0, cos yaw)`, Y up) at `pitch`
/// points.
pub fn facing(yaw: f32, pitch: f32) -> Vec3 {
    Vec3::new(
        yaw.sin() * pitch.cos(),
        pitch.sin(),
        yaw.cos() * pitch.cos(),
    )
}

impl Glider {
    /// Take off from the ground moving at `ground_velocity`: straight up at `LAUNCH_SPEED`
    /// on top of it.
    pub fn launch(ground_velocity: Vec3) -> Self {
        Self {
            velocity: ground_velocity.with_y(LAUNCH_SPEED),
            pitch: 0.0,
            gliding: false,
        }
    }

    pub fn speed(&self) -> f32 {
        self.velocity.length()
    }

    /// Fly `dt` seconds facing `yaw`, pitching toward `target_pitch` (the steered view;
    /// `None` keeps the pitch), with `height` yards of air under the mount. The displacement.
    pub fn step(
        &mut self,
        capability: &FlightCapability,
        yaw: f32,
        target_pitch: Option<f32>,
        height: f32,
        dt: f32,
    ) -> Vec3 {
        if let Some(target) = target_pitch {
            let turn = (target - self.pitch).clamp(
                -capability.pitching_rate_down * dt,
                capability.pitching_rate_up * dt,
            );
            self.pitch += turn;
        }
        if !self.gliding && self.velocity.y <= 0.0 && height >= capability.glide_start_min_height {
            self.gliding = true;
        }
        let before = self.velocity;
        let speed = before.length();
        let mut velocity = before;
        if self.gliding {
            velocity = lift(velocity, facing(yaw, self.pitch), capability, dt);
        }
        velocity.y -= GRAVITY * dt;
        let accelerated = velocity.length();
        let slowed = (accelerated - capability.air_friction * dt)
            .max(0.0)
            .min(capability.max_vel.max(speed));
        self.velocity = velocity.normalize_or_zero() * slowed;
        (before + self.velocity) * 0.5 * dt
    }
}

/// `velocity` turned toward `facing` by at most `lift_coefficient` × speed × `dt` radians.
fn lift(velocity: Vec3, facing: Vec3, capability: &FlightCapability, dt: f32) -> Vec3 {
    let speed = velocity.length();
    let Some(direction) = velocity.try_normalize() else {
        return velocity;
    };
    let angle = direction.angle_between(facing);
    let turn = (capability.lift_coefficient * speed * dt).min(angle);
    if angle <= f32::EPSILON {
        return velocity;
    }
    let axis = direction.cross(facing).try_normalize().unwrap_or(Vec3::Y);
    bevy::math::Quat::from_axis_angle(axis, turn) * direction * speed
}

/// The most a skyrider moving at `speed` can move at after falling `drop` yards (a climb is
/// a negative drop): energy is conserved at best, and gravity stops at `max_vel`. `None`
/// when the climb takes more energy than `speed` has.
pub fn speed_after(capability: &FlightCapability, speed: f32, drop: f32) -> Option<f32> {
    let squared = speed * speed + 2.0 * GRAVITY * drop;
    (squared >= 0.0).then(|| squared.sqrt().min(capability.max_vel.max(speed)))
}

/// The highest a skyrider moving at `speed` can climb, yards.
pub fn max_climb(speed: f32) -> f32 {
    speed * speed / (2.0 * GRAVITY)
}

#[cfg(test)]
#[path = "skyriding_tests.rs"]
mod tests;
