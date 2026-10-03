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
//! - gravity never pushes the speed past `max_vel`; above it (after an impulse) the mount
//!   also loses `over_max_deceleration` yards a second every second.
//!
//! The Skyriding abilities (`Glider::cast`) are DUMMY effects whose movement Retail runs in
//! an unpublished server script (Wowhead 372608: "Server-side script"; TrinityCore a352b1fa
//! defines `SMSG_MOVE_ADD_IMPULSE` but never sends it). Their impulses are the assumed
//! values below, capped by the published `AddImpulseMaxSpeed`; Aerial Halt is its DB2 aura.
//!
//! Lift and friction never add energy, so a skyrider's speed after falling `drop` yards is
//! at most `sqrt(v0² + 2·GRAVITY·drop)` (`speed_after`), the server's bound.

use bevy::math::Vec3;

/// `Movement::gravity` (TrinityCore MovementTypedefs.h:81), yards/s².
pub const GRAVITY: f32 = 19.291_103;

/// Surge Forward ("Flap forward."): a Skyriding Charge, impulse along the facing.
pub const SURGE_FORWARD: u32 = 372_608;
/// Skyward Ascent ("Flap upward."): a Skyriding Charge, impulse straight up.
pub const SKYWARD_ASCENT: u32 = 372_610;
/// Whirling Surge ("Spiral forward a great distance, increasing speed."): no charge, 30 s
/// category cooldown; impulse along the facing (the spiral is visual only).
pub const WHIRLING_SURGE: u32 = 361_584;
/// Aerial Halt ("Flap back, reducing forward movement."): APPLY_AURA
/// MOD_ADV_FLYING_AIR_FRICTION 513 at 10000% for SpellDuration 327 (500 ms).
pub const AERIAL_HALT: u32 = 403_092;

// --- Assumed values: unpublished, tune here (listed in game-server docs/specs/skyriding.md).
// Retail runs these movements in server scripts (Wowhead 372608: "Server-side script");
// only Skyward Ascent / Lift Off 374763 effect 0 DUMMY carries a number, 450.

/// Upward speed of a takeoff and of Skyward Ascent, yards/s: the 450 read as 450% of
/// `BASE_MOVEMENT_SPEED` 7 (PaperDollFrame.lua:38).
pub const LAUNCH_SPEED: f32 = 4.5 * 7.0;
/// Surge Forward's impulse along the facing, yards/s: the same 450% reading (its own DUMMY
/// carries no base points).
pub const SURGE_SPEED: f32 = 4.5 * 7.0;
/// Whirling Surge's impulse along the facing, yards/s: Surge Forward's (its DUMMY aura
/// carries no base points).
pub const WHIRLING_SURGE_SPEED: f32 = 4.5 * 7.0;

// --- End of assumed values.

/// Aerial Halt's air friction multiplier: `ApplyPct(AirFriction, 10000)` (TC
/// `Unit::UpdateAdvFlyingSpeed`, Unit.cpp:9080).
pub const HALT_FRICTION_FACTOR: f32 = 10_000.0 / 100.0;
/// Aerial Halt's aura duration, seconds (SpellDuration 327).
pub const HALT_SECS: f32 = 0.5;

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
    /// `AddImpulseMaxSpeed`: the most an impulse brings the mount to, yards/s.
    pub add_impulse_max_speed: f32,
    /// `OverMaxDeceleration`: speed lost per second above `max_vel`, yards/s².
    pub over_max_deceleration: f32,
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
    add_impulse_max_speed: 100.0,
    over_max_deceleration: 7.0,
};

/// A skyrider in the air.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Glider {
    pub velocity: Vec3,
    /// The mount's pitch, radians (+ up).
    pub pitch: f32,
    /// Wings out: lift steers the velocity.
    pub gliding: bool,
    /// Seconds left of Aerial Halt's air friction.
    pub halt_secs: f32,
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
            halt_secs: 0.0,
        }
    }

    pub fn speed(&self) -> f32 {
        self.velocity.length()
    }

    /// Resolve the Skyriding ability `spell_id` cast facing `yaw`; `false` for any other
    /// spell.
    pub fn cast(&mut self, capability: &FlightCapability, spell_id: u32, yaw: f32) -> bool {
        match spell_id {
            SURGE_FORWARD => self.impulse(capability, facing(yaw, self.pitch) * SURGE_SPEED),
            WHIRLING_SURGE => {
                self.impulse(capability, facing(yaw, self.pitch) * WHIRLING_SURGE_SPEED);
            }
            SKYWARD_ASCENT => self.impulse(capability, Vec3::Y * LAUNCH_SPEED),
            AERIAL_HALT => self.halt_secs = HALT_SECS,
            _ => return false,
        }
        true
    }

    /// Add `impulse` to the velocity, no faster than `add_impulse_max_speed` (or the speed
    /// before, when already past it).
    fn impulse(&mut self, capability: &FlightCapability, impulse: Vec3) {
        let cap = capability.add_impulse_max_speed.max(self.speed());
        self.velocity = (self.velocity + impulse).clamp_length_max(cap);
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
        let halted = self.halt_secs.min(dt);
        self.halt_secs -= halted;
        let friction = capability.air_friction * (dt + (HALT_FRICTION_FACTOR - 1.0) * halted);
        let mut slowed = (accelerated - friction)
            .max(0.0)
            .min(capability.max_vel.max(speed));
        if slowed > capability.max_vel {
            slowed = (slowed - capability.over_max_deceleration * dt).max(capability.max_vel);
        }
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

/// The most a skyrider moving at `speed` moves at after an ability's impulse of
/// `impulse_speed`: `|v + i| ≤ |v| + |i|`, capped like `Glider::cast`.
pub fn impulse_bound(capability: &FlightCapability, speed: f32, impulse_speed: f32) -> f32 {
    (speed + impulse_speed).min(capability.add_impulse_max_speed.max(speed))
}

/// The impulse speed of the Skyriding ability `spell_id`, if it has one.
pub fn ability_impulse(spell_id: u32) -> Option<f32> {
    match spell_id {
        SURGE_FORWARD => Some(SURGE_SPEED),
        WHIRLING_SURGE => Some(WHIRLING_SURGE_SPEED),
        SKYWARD_ASCENT => Some(LAUNCH_SPEED),
        _ => None,
    }
}

/// The highest a skyrider moving at `speed` can climb, yards.
pub fn max_climb(speed: f32) -> f32 {
    speed * speed / (2.0 * GRAVITY)
}

#[cfg(test)]
#[path = "skyriding_tests.rs"]
mod tests;
