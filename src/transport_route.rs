//! Looping transport route math for constant-speed, no-stop taxi paths.
//!
//! Mirrors TrinityCore a352b1fa `TransportMgr.cpp` (`SplineRawInitializer`,
//! `InitializeLeg`, `TransportTemplate::ComputePosition`) together with
//! `Movement::Spline<double>` (`Spline.cpp`, `SplineImpl.h`) and the
//! `Position::RelocateOffset` / `GetPositionOffsetTo` passenger transforms.
//! Pause (`TAXI_PATH_NODE_FLAG_STOP`), teleport and event nodes are outside
//! this model; callers validate them before building a route.

use std::f32::consts::PI;

/// `InitializeLeg` calls `set_steps_per_segment(20)` before `initLengths()`.
pub const STEPS_PER_SEGMENT: u32 = 20;

/// Row-major `s_catmullRomCoeffs` from `Spline.cpp`.
const CATMULL_ROM: [[f32; 4]; 4] = [
    [-0.5, 1.5, -1.5, 0.5],
    [1.0, -2.5, 2.0, -0.5],
    [-0.5, 0.0, 0.5, 0.0],
    [0.0, 1.0, 0.0, 0.0],
];

/// WoW-space position (Z up) plus orientation in radians, `[0, 2π)`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TransportPose {
    pub position: [f32; 3],
    pub orientation: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TransportRouteError {
    /// Catmull-Rom needs one tangent knot at each end plus one segment.
    TooFewKnots {
        count: usize,
    },
    NonPositiveSpeed {
        speed: f32,
    },
    /// `uint32(length / speed * 1000)` truncated to zero milliseconds.
    ZeroPeriod,
}

#[derive(Debug, Clone)]
pub struct TransportRoute {
    knots: Vec<[f32; 3]>,
    /// Cumulative spline length at knot `i`; entries 0 and 1 are zero.
    lengths: Vec<f64>,
    speed: f64,
    period_ms: u32,
}

impl TransportRoute {
    /// Builds a noncyclic Catmull-Rom route over ordered taxi path knots.
    /// The first and last knots only shape tangents: travel runs from
    /// `knots[1]` to `knots[len - 2]` and then restarts.
    pub fn new(knots: Vec<[f32; 3]>, speed: f32) -> Result<Self, TransportRouteError> {
        if knots.len() < 4 {
            return Err(TransportRouteError::TooFewKnots { count: knots.len() });
        }
        if speed.is_nan() || speed <= 0.0 {
            return Err(TransportRouteError::NonPositiveSpeed { speed });
        }
        let lengths = segment_lengths(&knots);
        let speed = f64::from(speed);
        let period_ms = (lengths[knots.len() - 2] / speed * 1000.0) as u32;
        if period_ms == 0 {
            return Err(TransportRouteError::ZeroPeriod);
        }
        Ok(Self {
            knots,
            lengths,
            speed,
            period_ms,
        })
    }

    pub fn period_ms(&self) -> u32 {
        self.period_ms
    }

    pub fn length(&self) -> f64 {
        self.lengths[self.index_hi()]
    }

    /// `ComputePosition(time)`: position and facing after `elapsed_ms`,
    /// looping every `period_ms`.
    pub fn pose_at(&self, elapsed_ms: u64) -> TransportPose {
        let time_ms = elapsed_ms % u64::from(self.period_ms);
        let distance = time_ms as f64 * 0.001 * self.speed;
        let percent = (distance / self.length()).min(1.0) as f32;
        let (index, u) = self.compute_index(percent);
        let control = self.control_points(index);
        let position = evaluate(control, [u * u * u, u * u, u, 1.0]);
        let direction = evaluate(control, [3.0 * u * u, 2.0 * u, 1.0, 0.0]);
        TransportPose {
            position,
            orientation: normalize_orientation(direction[1].atan2(direction[0]) + PI),
        }
    }

    fn index_hi(&self) -> usize {
        self.knots.len() - 2
    }

    /// `Spline::computeIndex`: the segment containing `percent` of the
    /// length and the parameter `u`, linear in that segment's length.
    fn compute_index(&self, percent: f32) -> (usize, f32) {
        let length = f64::from(percent) * self.length();
        let mut index = 1;
        while index + 1 < self.index_hi() && self.lengths[index + 1] < length {
            index += 1;
        }
        let segment = (self.lengths[index + 1] - self.lengths[index]) as f32;
        let u = (length - self.lengths[index]) / f64::from(segment);
        (index, u as f32)
    }

    fn control_points(&self, index: usize) -> &[[f32; 3]] {
        &self.knots[index - 1..index + 3]
    }
}

impl TransportPose {
    /// `Position::GetPositionWithOffset`: transport-local offset to world.
    pub fn local_to_world(&self, local: TransportPose) -> TransportPose {
        let (sin, cos) = self.orientation.sin_cos();
        let [x, y, z] = local.position;
        TransportPose {
            position: [
                self.position[0] + x * cos - y * sin,
                self.position[1] + y * cos + x * sin,
                self.position[2] + z,
            ],
            orientation: normalize_orientation(self.orientation + local.orientation),
        }
    }

    /// `Position::GetPositionOffsetTo`: world pose to transport-local offset.
    pub fn world_to_local(&self, world: TransportPose) -> TransportPose {
        let (sin, cos) = self.orientation.sin_cos();
        let dx = world.position[0] - self.position[0];
        let dy = world.position[1] - self.position[1];
        TransportPose {
            position: [
                dx * cos + dy * sin,
                dy * cos - dx * sin,
                world.position[2] - self.position[2],
            ],
            orientation: normalize_orientation(world.orientation - self.orientation),
        }
    }
}

/// `Position::NormalizeOrientation`: wraps radians into `[0, 2π)`.
pub fn normalize_orientation(orientation: f32) -> f32 {
    let wrapped = orientation.rem_euclid(2.0 * PI);
    // rem_euclid can round a tiny negative input up to exactly 2π.
    if wrapped >= 2.0 * PI { 0.0 } else { wrapped }
}

/// `Spline::initLengths` with `SegLengthCatmullRom`: cumulative lengths for
/// `index_lo = 1 ..= index_hi = len - 2`, each segment summed in float over
/// `STEPS_PER_SEGMENT` chords.
fn segment_lengths(knots: &[[f32; 3]]) -> Vec<f64> {
    let index_hi = knots.len() - 2;
    let mut lengths = vec![0.0; index_hi + 1];
    let mut total = 0.0;
    for index in 1..index_hi {
        let control = &knots[index - 1..index + 3];
        let mut current = control[1];
        let mut segment = 0.0f32;
        for step in 1..=STEPS_PER_SEGMENT {
            let t = step as f32 / STEPS_PER_SEGMENT as f32;
            let next = evaluate(control, [t * t * t, t * t, t, 1.0]);
            segment += distance(current, next);
            current = next;
        }
        total += f64::from(segment);
        lengths[index + 1] = total;
    }
    lengths
}

/// `C_Evaluate` / `C_Evaluate_Derivative`: `powers * CATMULL_ROM` weights
/// applied to four control points.
fn evaluate(control: &[[f32; 3]], powers: [f32; 4]) -> [f32; 3] {
    let mut result = [0.0; 3];
    for (column, point) in control.iter().enumerate() {
        let weight: f32 = (0..4)
            .map(|row| powers[row] * CATMULL_ROM[row][column])
            .sum();
        for axis in 0..3 {
            result[axis] += point[axis] * weight;
        }
    }
    result
}

fn distance(a: [f32; 3], b: [f32; 3]) -> f32 {
    let delta = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    (delta[0] * delta[0] + delta[1] * delta[1] + delta[2] * delta[2]).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// TaxiPathNode.csv 12.1.0.69933 PathID 6519 (map 1803), NodeIndex 0..=14.
    const PATH_6519: [[f32; 3]; 15] = [
        [1261.6597900391, 2588.9306640625, 297.0866394043],
        [1193.90625, 2677.03125, 297.0866394043],
        [1179.8090820312, 2785.4948730469, 297.08862304688],
        [1235.53125, 2874.0051269531, 297.08862304688],
        [1301.5416259766, 2915.9306640625, 297.08862304688],
        [1394.6771240234, 2934.8352050781, 297.08862304688],
        [1479.2673339844, 2900.2275390625, 297.0866394043],
        [1553.9132080078, 2812.0400390625, 297.08862304688],
        [1580.9514160156, 2705.2570800781, 297.0866394043],
        [1518.7153320312, 2588.7778320312, 297.08862304688],
        [1440.0660400391, 2554.0122070312, 297.08862304688],
        [1343.7014160156, 2543.9289550781, 297.08862304688],
        [1261.65625, 2588.9306640625, 297.0866394043],
        [1193.9097900391, 2677.0346679688, 297.08517456055],
        [1179.8055419922, 2785.4982910156, 297.08517456055],
    ];

    /// TaxiPathNode.csv 12.1.0.69933 PathID 6520 (map 1803), NodeIndex 0..=14.
    const PATH_6520: [[f32; 3]; 15] = [
        [1479.2700195312, 2900.2299804688, 297.08999633789],
        [1553.9100341797, 2812.0400390625, 297.08517456055],
        [1580.9499511719, 2705.2600097656, 297.08517456055],
        [1518.7199707031, 2588.7800292969, 297.08999633789],
        [1440.0699462891, 2554.0100097656, 297.08517456055],
        [1343.6999511719, 2543.9299316406, 297.08517456055],
        [1261.6600341797, 2588.9299316406, 297.08517456055],
        [1193.9100341797, 2677.0300292969, 297.08517456055],
        [1179.8100585938, 2785.5, 297.08517456055],
        [1235.5300292969, 2874.0100097656, 297.08517456055],
        [1301.5400390625, 2915.9299316406, 297.08517456055],
        [1394.6800537109, 2934.8400878906, 297.08517456055],
        [1479.2700195312, 2900.2299804688, 297.08517456055],
        [1553.9100341797, 2812.0400390625, 297.08517456055],
        [1580.9499511719, 2705.2600097656, 297.08517456055],
    ];

    fn assert_close(actual: f32, expected: f32, tolerance: f32) {
        assert!(
            (actual - expected).abs() <= tolerance,
            "{actual} != {expected} (±{tolerance})"
        );
    }

    fn assert_position(actual: [f32; 3], expected: [f32; 3], tolerance: f32) {
        for axis in 0..3 {
            assert_close(actual[axis], expected[axis], tolerance);
        }
    }

    fn straight_route() -> TransportRoute {
        let knots = vec![
            [0.0, 0.0, 5.0],
            [10.0, 0.0, 5.0],
            [20.0, 0.0, 5.0],
            [30.0, 0.0, 5.0],
        ];
        TransportRoute::new(knots, 2.0).unwrap()
    }

    #[test]
    fn straight_line_travels_only_between_inner_knots_at_constant_speed() {
        let route = straight_route();
        assert_close(route.length() as f32, 10.0, 1e-4);
        assert_eq!(route.period_ms(), 5000);

        let start = route.pose_at(0);
        assert_position(start.position, [10.0, 0.0, 5.0], 1e-4);
        let midpoint = route.pose_at(2500);
        assert_position(midpoint.position, [15.0, 0.0, 5.0], 1e-4);
        // atan2(0, +x) + PI: the source faces opposite the travel direction.
        assert_close(midpoint.orientation, PI, 1e-6);
        assert_position(route.pose_at(4999).position, [19.998, 0.0, 5.0], 1e-3);
    }

    #[test]
    fn elapsed_time_wraps_at_whole_millisecond_period() {
        let route = straight_route();
        let period = u64::from(route.period_ms());
        assert_eq!(route.pose_at(period), route.pose_at(0));
        assert_eq!(route.pose_at(period * 3 + 1250), route.pose_at(1250));
        assert_position(
            route.pose_at(period + 1250).position,
            [12.5, 0.0, 5.0],
            1e-4,
        );
    }

    #[test]
    fn period_truncates_length_over_speed_to_milliseconds() {
        // Length 10 at speed 3 is 3333.33 ms; uint32() truncates to 3333.
        let knots = vec![
            [0.0, 0.0, 0.0],
            [0.0, 10.0, 0.0],
            [0.0, 20.0, 0.0],
            [0.0, 30.0, 0.0],
        ];
        let route = TransportRoute::new(knots, 3.0).unwrap();
        assert_eq!(route.period_ms(), 3333);
        // Travel along +Y: atan2(1, 0) + PI = 3PI/2.
        assert_close(route.pose_at(1000).orientation, 1.5 * PI, 1e-6);
        assert_position(route.pose_at(1000).position, [0.0, 13.0, 0.0], 1e-4);
    }

    #[test]
    fn segment_parameter_is_linear_in_measured_segment_length() {
        // Uneven knot spacing: segment two (20 -> 40) is twice as long as
        // segment one (10 -> 20), so 15 yards in is u = 5/20 of segment two,
        // not the arc-length point x = 25.
        let knots = vec![
            [0.0, 0.0, 0.0],
            [10.0, 0.0, 0.0],
            [20.0, 0.0, 0.0],
            [40.0, 0.0, 0.0],
            [60.0, 0.0, 0.0],
        ];
        let route = TransportRoute::new(knots, 1.0).unwrap();
        assert_close(route.length() as f32, 30.0, 1e-3);
        assert_position(route.pose_at(10_000).position, [20.0, 0.0, 0.0], 1e-3);
        let u: f32 = 5.0 / route_segment_two_length(&route);
        assert_close(u, 0.25, 1e-4);
        let expected_x = catmull_rom_x([10.0, 20.0, 40.0, 60.0], u);
        assert!((expected_x - 25.0).abs() > 0.1);
        assert_close(route.pose_at(15_000).position[0], expected_x, 1e-3);
    }

    fn route_segment_two_length(route: &TransportRoute) -> f32 {
        (route.lengths[3] - route.lengths[2]) as f32
    }

    fn catmull_rom_x(p: [f32; 4], u: f32) -> f32 {
        let t = [u * u * u, u * u, u, 1.0];
        let mut x = 0.0;
        for column in 0..4 {
            let weight: f32 = (0..4).map(|row| t[row] * CATMULL_ROM[row][column]).sum();
            x += weight * p[column];
        }
        x
    }

    #[test]
    fn seething_route_6519_starts_at_node_one_with_source_tangent_yaw() {
        let route = TransportRoute::new(PATH_6519.to_vec(), 20.0).unwrap();
        let start = route.pose_at(0);
        assert_position(start.position, PATH_6519[1], 1e-3);
        // Derivative at u=0 is 0.5 * (knot2 - knot0).
        let dx = PATH_6519[2][0] - PATH_6519[0][0];
        let dy = PATH_6519[2][1] - PATH_6519[0][1];
        assert_close(
            start.orientation,
            normalize_orientation(dy.atan2(dx) + PI),
            1e-5,
        );
        assert_close(start.orientation, 5.107, 1e-3);
    }

    #[test]
    fn seething_route_6519_loops_back_to_node_thirteen_before_wrapping() {
        let route = TransportRoute::new(PATH_6519.to_vec(), 20.0).unwrap();
        let period = route.period_ms();
        assert!(period > 0);
        let last = route.pose_at(u64::from(period) - 1);
        // One millisecond at speed 20 is 0.02 yards short of node 13.
        assert_position(last.position, PATH_6519[13], 0.05);
        assert_eq!(route.pose_at(u64::from(period)), route.pose_at(0));
    }

    #[test]
    fn seething_routes_6519_and_6520_share_the_same_loop_offset_by_nodes() {
        let alliance = TransportRoute::new(PATH_6519.to_vec(), 20.0).unwrap();
        let horde = TransportRoute::new(PATH_6520.to_vec(), 20.0).unwrap();
        assert_position(horde.pose_at(0).position, PATH_6520[1], 1e-3);
        // 6520 starts at 6519 node 7, so both trace the same geometry.
        assert_position(PATH_6520[1], PATH_6519[7], 0.01);
        assert_close(alliance.length() as f32, horde.length() as f32, 0.5);
        assert_eq!(
            horde.period_ms() / 100,
            alliance.period_ms() / 100,
            "same loop length within 100 ms"
        );
    }

    #[test]
    fn rejects_routes_without_a_travel_segment_or_speed() {
        let three = vec![[0.0; 3], [1.0, 0.0, 0.0], [2.0, 0.0, 0.0]];
        assert_eq!(
            TransportRoute::new(three, 1.0).unwrap_err(),
            TransportRouteError::TooFewKnots { count: 3 }
        );
        let four = vec![[0.0; 3], [1.0, 0.0, 0.0], [2.0, 0.0, 0.0], [3.0, 0.0, 0.0]];
        assert_eq!(
            TransportRoute::new(four.clone(), 0.0).unwrap_err(),
            TransportRouteError::NonPositiveSpeed { speed: 0.0 }
        );
        assert_eq!(
            TransportRoute::new(four, 10_000.0).unwrap_err(),
            TransportRouteError::ZeroPeriod
        );
    }

    #[test]
    fn local_offset_rotates_by_transport_yaw_and_adds_facing() {
        let transport = TransportPose {
            position: [100.0, 200.0, 5.0],
            orientation: 0.5 * PI,
        };
        let local = TransportPose {
            position: [2.0, 1.0, 3.0],
            orientation: 0.25,
        };
        let world = transport.local_to_world(local);
        assert_position(world.position, [99.0, 202.0, 8.0], 1e-4);
        assert_close(world.orientation, 0.5 * PI + 0.25, 1e-6);
    }

    #[test]
    fn world_and_local_round_trip_preserves_position_and_facing() {
        let transport = TransportRoute::new(PATH_6519.to_vec(), 20.0)
            .unwrap()
            .pose_at(12_345);
        let local = TransportPose {
            position: [-4.5, 7.25, 1.5],
            orientation: 6.0,
        };
        let world = transport.local_to_world(local);
        let back = transport.world_to_local(world);
        assert_position(back.position, local.position, 1e-3);
        assert_close(back.orientation, local.orientation, 1e-4);

        let world_pose = TransportPose {
            position: [1300.0, 2700.0, 300.0],
            orientation: 0.1,
        };
        let again = transport.local_to_world(transport.world_to_local(world_pose));
        assert_position(again.position, world_pose.position, 1e-3);
        assert_close(again.orientation, world_pose.orientation, 1e-4);
    }

    #[test]
    fn orientation_normalizes_into_zero_to_two_pi() {
        assert_close(normalize_orientation(-0.5), 2.0 * PI - 0.5, 1e-6);
        assert_close(normalize_orientation(2.0 * PI + 0.25), 0.25, 1e-5);
        assert_close(normalize_orientation(1.0), 1.0, 0.0);
    }
}
