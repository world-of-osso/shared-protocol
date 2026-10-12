//! Exact ground Walk paths (TDB Flags2): consume authored linear waypoints.
pub(super) fn distance(a: [f32; 3], b: [f32; 3]) -> f32 {
    a.iter()
        .zip(b)
        .map(|(a, b)| (a - b) * (a - b))
        .sum::<f32>()
        .sqrt()
}

pub(super) fn follow_linear_path(
    position: &mut [f32; 3],
    next_node: &mut usize,
    points: &[[f32; 3]],
    distance_left: &mut f32,
) -> bool {
    while let Some(&target) = points.get(*next_node) {
        let remaining = distance(*position, target);
        if remaining > *distance_left {
            let fraction = *distance_left / remaining;
            *position = std::array::from_fn(|axis| {
                position[axis] + (target[axis] - position[axis]) * fraction
            });
            *distance_left = 0.0;
            return false;
        }
        *position = target;
        *distance_left -= remaining;
        *next_node += 1;
    }
    true
}
