use bevy::math::Vec3;

use super::{BODY_RADIUS, Body};

/// The human male character display's (57899) Stand eye point in the map 0 bake.
const HUMAN_EYE: [f32; 3] = [0.149, 0.001, 1.897];

#[test]
fn eyes_turn_with_the_facing() {
    let feet = Vec3::new(-8913.0, 101.5, -553.0);
    // Yaw 0 faces Bevy +Z; WoW left (+Y) is then Bevy +X.
    let ahead = Body::placed(feet, 0.0, HUMAN_EYE);
    assert!(ahead.eyes.distance(feet + Vec3::new(0.001, 1.897, 0.149)) < 1e-4);
    // A quarter turn: forward is Bevy +X; WoW left (+Y) is then Bevy -Z.
    let turned = Body::placed(feet, std::f32::consts::FRAC_PI_2, [0.0, 1.0, 1.0]);
    assert!(turned.eyes.distance(feet + Vec3::new(0.0, 1.0, -1.0)) < 1e-4);
}

#[test]
fn the_volume_is_a_capsule_around_feet_to_eyes() {
    let body = Body::placed(Vec3::ZERO, 0.0, HUMAN_EYE);
    for point in body.aim_points() {
        assert!(body.contains(point, 0.0));
    }
    let beside_chest = body.chest() + Vec3::X * (BODY_RADIUS + 0.1);
    assert!(!body.contains(beside_chest, 0.0));
    assert!(body.contains(beside_chest, 0.2));
    let over_head = body.eyes + Vec3::Y * (BODY_RADIUS + 0.5);
    assert!(!body.contains(over_head, 0.0));
}
