use super::*;

const DT: f32 = 1.0 / 60.0;
/// High enough that the ground never matters.
const HIGH: f32 = 1000.0;

fn glider(speed: f32, pitch: f32) -> Glider {
    Glider {
        velocity: facing(0.0, pitch) * speed,
        pitch,
        gliding: true,
        halt_secs: 0.0,
    }
}

/// Fly `secs` at `HIGH` steering toward `pitch`; the height lost.
fn fly(glider: &mut Glider, pitch: f32, secs: f32) -> f32 {
    let steps = (secs / DT).round() as u32;
    (0..steps)
        .map(|_| -glider.step(&SKYRIDING, 0.0, Some(pitch), HIGH, DT).y)
        .sum()
}

#[test]
fn a_launch_rises_without_lift_to_its_apex_then_spreads_its_wings() {
    let mut glider = Glider::launch(Vec3::ZERO);
    let mut height = 0.0;
    let mut apex = 0.0_f32;
    while !glider.gliding {
        height += glider.step(&SKYRIDING, 0.0, None, height, DT).y;
        apex = apex.max(height);
        assert!(
            glider.velocity.x == 0.0 && glider.velocity.z == 0.0,
            "no lift before gliding"
        );
    }
    // Decelerated by gravity 19.29 + friction 1.5 from 31.5 yd/s: 31.5² / (2 × 20.79).
    assert!((apex - 23.86).abs() < 0.1, "apex {apex}");
    assert!(height >= SKYRIDING.glide_start_min_height);
}

#[test]
fn a_launch_too_low_to_glide_falls() {
    let mut glider = Glider::launch(Vec3::ZERO);
    // Ground 20 yards up hill under the whole climb: never 7.5 yards of air.
    for _ in 0..200 {
        glider.step(&SKYRIDING, 0.0, None, 5.0, DT);
    }
    assert!(!glider.gliding);
    // Up for 31.5 / 20.79 = 1.52 s, then down at 19.29 − 1.5 for the other 1.82 s.
    assert!(
        (glider.velocity.y + 32.4).abs() < 0.5,
        "{:?}",
        glider.velocity
    );
}

#[test]
fn diving_turns_height_into_speed_up_to_max_vel() {
    let mut diver = glider(20.0, 0.0);
    let dive = -60.0_f32.to_radians();
    let drop = fly(&mut diver, dive, 2.0);
    let speed = diver.speed();
    // Gravity along a 60° dive less friction: 19.29 × sin 60° − 1.5 = 15.2 yd/s².
    assert!(speed > 45.0 && speed < 50.0, "speed {speed}");
    let bound = speed_after(&SKYRIDING, 20.0, drop).unwrap();
    assert!(speed <= bound, "{speed} over the energy bound {bound}");

    fly(&mut diver, dive, 5.0);
    assert_eq!(diver.speed(), SKYRIDING.max_vel);
}

#[test]
fn climbing_bleeds_speed_until_the_mount_stalls() {
    let mut climber = glider(60.0, 30.0_f32.to_radians());
    let drop = fly(&mut climber, 30.0_f32.to_radians(), 2.0);
    let speed = climber.speed();
    // 19.29 × sin 30° + 1.5 = 11.1 yd/s² lost: about 37.7 yd/s after 2 s.
    assert!(speed > 36.0 && speed < 40.0, "speed {speed}");
    // About 49 yd/s on average up a 30° slope for 2 s.
    assert!(drop < -45.0, "climbed {}", -drop);
    assert!(speed <= speed_after(&SKYRIDING, 60.0, drop).unwrap());

    let up = 30.0_f32.to_radians();
    let mut climbed = -drop;
    let mut sinking = false;
    for _ in 0..(6.0 / DT) as u32 {
        climbed += climber.step(&SKYRIDING, 0.0, Some(up), HIGH, DT).y;
        sinking |= climber.velocity.y < 0.0;
    }
    assert!(
        climbed <= max_climb(60.0),
        "{climbed} over {}",
        max_climb(60.0)
    );
    // Below sqrt(g / 0.07) = 16.6 yd/s lift loses to gravity.
    assert!(
        sinking && climber.speed() < 16.6,
        "never stalled: {:?}",
        climber.velocity
    );
}

#[test]
fn a_level_glide_holds_its_height_and_slows_by_air_friction() {
    let mut level = glider(50.0, 0.0);
    let drop = fly(&mut level, 0.0, 4.0);
    // Lift at 0.07 × speed rad/s holds the line above sqrt(g / 0.07) = 16.6 yd/s.
    assert!(drop.abs() < 1.5, "lost {drop} yards");
    assert!(
        (level.speed() - (50.0 - 1.5 * 4.0)).abs() < 0.5,
        "{}",
        level.speed()
    );
}

#[test]
fn no_flight_beats_the_energy_bound() {
    for pitch in [-90.0_f32, -45.0, -10.0, 0.0, 15.0, 45.0, 80.0] {
        let mut flyer = glider(30.0, 0.0);
        let mut drop = 0.0;
        for _ in 0..600 {
            drop -= flyer
                .step(&SKYRIDING, 0.3, Some(pitch.to_radians()), HIGH, DT)
                .y;
            let bound = speed_after(&SKYRIDING, 30.0, drop).unwrap();
            assert!(
                flyer.speed() <= bound + 1e-3,
                "{pitch}°: {} over {bound}",
                flyer.speed()
            );
        }
    }
}

#[test]
fn the_energy_bound_has_concrete_numbers() {
    // sqrt(30² + 2 × 19.2911 × 20) = 40.89.
    assert!((speed_after(&SKYRIDING, 30.0, 20.0).unwrap() - 40.886).abs() < 0.01);
    // Gravity stops at MaxVel 65.
    assert_eq!(speed_after(&SKYRIDING, 60.0, 500.0), Some(65.0));
    // 30 yd/s climbs at most 30² / 38.58 = 23.33 yards.
    assert!((max_climb(30.0) - 23.327).abs() < 0.01);
    assert_eq!(speed_after(&SKYRIDING, 30.0, -24.0), None);
}

#[test]
fn surge_forward_flaps_along_the_facing() {
    let mut flyer = glider(30.0, 0.0);
    assert!(flyer.cast(&SKYRIDING, SURGE_FORWARD, 0.0));
    // 30 + 31.5 yd/s straight ahead (yaw 0 faces +Z).
    assert!(
        (flyer.velocity - Vec3::new(0.0, 0.0, 61.5)).length() < 1e-3,
        "{:?}",
        flyer.velocity
    );
    let speed = flyer.speed();
    assert!(speed <= impulse_bound(&SKYRIDING, 30.0, SURGE_SPEED) + 1e-3);
}

#[test]
fn skyward_ascent_flaps_upward() {
    let mut flyer = glider(30.0, 0.0);
    assert!(flyer.cast(&SKYRIDING, SKYWARD_ASCENT, 1.0));
    assert!(
        (flyer.velocity - Vec3::new(0.0, 31.5, 30.0)).length() < 1e-3,
        "{:?}",
        flyer.velocity
    );
    // |v + up| ≤ |v| + 31.5: the server's bound holds.
    assert!(flyer.speed() <= impulse_bound(&SKYRIDING, 30.0, LAUNCH_SPEED));
}

#[test]
fn an_impulse_stops_at_add_impulse_max_speed() {
    let mut flyer = glider(90.0, 0.0);
    flyer.cast(&SKYRIDING, SURGE_FORWARD, 0.0);
    assert!((flyer.speed() - 100.0).abs() < 1e-3, "{}", flyer.speed());
    assert_eq!(impulse_bound(&SKYRIDING, 90.0, SURGE_SPEED), 100.0);
    // Already past it (no published path, but the bound never shrinks a speed).
    assert_eq!(impulse_bound(&SKYRIDING, 120.0, SURGE_SPEED), 120.0);
}

#[test]
fn over_max_vel_the_mount_decelerates_by_over_max_deceleration() {
    let mut flyer = glider(90.0, 0.0);
    fly(&mut flyer, 0.0, 1.0);
    // OverMaxDeceleration 7 on top of AirFriction 1.5.
    assert!((flyer.speed() - 81.5).abs() < 0.3, "{}", flyer.speed());
    fly(&mut flyer, 0.0, 4.0);
    // Down to MaxVel 65 after 25 / 8.5 = 2.9 s, then by air friction alone.
    assert!(
        (flyer.speed() - (65.0 - 1.5 * 2.06)).abs() < 0.5,
        "{}",
        flyer.speed()
    );
}

#[test]
fn aerial_halt_stops_forward_movement_for_half_a_second() {
    let mut flyer = glider(60.0, 0.0);
    assert!(flyer.cast(&SKYRIDING, AERIAL_HALT, 0.0));
    fly(&mut flyer, 0.0, 0.4);
    // AirFriction 1.5 × 10000% = 150 yd/s²: 60 yd/s is gone in 0.4 s.
    assert!(flyer.velocity.z.abs() < 1.0, "{:?}", flyer.velocity);
    fly(&mut flyer, 0.0, 0.2);
    assert_eq!(flyer.halt_secs, 0.0);
    // Falling again under plain air friction.
    let falling = flyer.speed();
    fly(&mut flyer, 0.0, 0.5);
    assert!(flyer.speed() > falling, "{} after {falling}", flyer.speed());
}

#[test]
fn other_spells_leave_the_glider_alone() {
    let mut flyer = glider(30.0, 0.0);
    assert!(!flyer.cast(&SKYRIDING, 32235, 0.0));
    assert_eq!(flyer, glider(30.0, 0.0));
}
