use std::sync::Arc;

use bevy::math::{Affine3A, Quat, Vec3};

use super::*;

const RENDER: u8 = 0x20;
const DETAIL: u8 = 0x04;
const LEAF: u16 = 0x04;
const AXIS_Y: u16 = 0x01;

/// One MOPY face in file coordinates (X, Y, Z up).
struct Face {
    corners: [[f32; 3]; 3],
    flags: u8,
    material: u8,
}

/// Two render faces covering the file-space square `[-10, 10]²` at height `z`.
fn floor(z: f32) -> [Face; 2] {
    quad([-10.0, 10.0], [-10.0, 10.0], z, RENDER, 0)
}

fn quad(x: [f32; 2], y: [f32; 2], z: f32, flags: u8, material: u8) -> [Face; 2] {
    let corners = [
        [x[0], y[0], z],
        [x[1], y[0], z],
        [x[1], y[1], z],
        [x[0], y[1], z],
    ];
    [
        Face {
            corners: [corners[0], corners[1], corners[2]],
            flags,
            material,
        },
        Face {
            corners: [corners[0], corners[2], corners[3]],
            flags,
            material,
        },
    ]
}

struct Node {
    flags: u16,
    neg: i16,
    pos: i16,
    faces: std::ops::Range<u32>,
    dist: f32,
}

fn leaf(faces: std::ops::Range<u32>) -> Node {
    Node {
        flags: LEAF,
        neg: -1,
        pos: -1,
        faces,
        dist: 0.0,
    }
}

fn chunk(tag: &[u8; 4], payload: &[u8]) -> Vec<u8> {
    let mut bytes: Vec<u8> = tag.iter().rev().copied().collect();
    bytes.extend_from_slice(&(payload.len() as u32).to_le_bytes());
    bytes.extend_from_slice(payload);
    bytes
}

fn f32s(values: impl IntoIterator<Item = f32>) -> Vec<u8> {
    values.into_iter().flat_map(f32::to_le_bytes).collect()
}

/// A WMO group file whose BSP nodes index `faces` in order (MOBR `i -> i`).
fn group_file(mogp_flags: u32, faces: &[Face], nodes: &[Node]) -> Vec<u8> {
    let corners: Vec<[f32; 3]> = faces.iter().flat_map(|face| face.corners).collect();
    let (min, max) = corners.iter().fold(
        (Vec3::splat(f32::MAX), Vec3::splat(f32::MIN)),
        |(min, max), corner| (min.min(Vec3::from(*corner)), max.max(Vec3::from(*corner))),
    );
    let mut header = vec![0u8; 0x44];
    header[8..12].copy_from_slice(&mogp_flags.to_le_bytes());
    header[12..36].copy_from_slice(&f32s(min.to_array().into_iter().chain(max.to_array())));
    let mopy: Vec<u8> = faces
        .iter()
        .flat_map(|face| [face.flags, face.material])
        .collect();
    let movi: Vec<u8> = (0..corners.len() as u16)
        .flat_map(u16::to_le_bytes)
        .collect();
    let movt = f32s(corners.iter().flatten().copied());
    let mobn: Vec<u8> = nodes
        .iter()
        .flat_map(|node| {
            let mut entry = Vec::with_capacity(16);
            entry.extend_from_slice(&node.flags.to_le_bytes());
            entry.extend_from_slice(&node.neg.to_le_bytes());
            entry.extend_from_slice(&node.pos.to_le_bytes());
            entry.extend_from_slice(&((node.faces.end - node.faces.start) as u16).to_le_bytes());
            entry.extend_from_slice(&node.faces.start.to_le_bytes());
            entry.extend_from_slice(&node.dist.to_le_bytes());
            entry
        })
        .collect();
    let mobr: Vec<u8> = (0..faces.len() as u16).flat_map(u16::to_le_bytes).collect();
    let mut mogp = header;
    for (tag, payload) in [
        (b"MOPY", mopy),
        (b"MOVI", movi),
        (b"MOVT", movt),
        (b"MOBN", mobn),
        (b"MOBR", mobr),
    ] {
        mogp.extend(chunk(tag, &payload));
    }
    let mut file = chunk(b"MVER", &17u32.to_le_bytes());
    file.extend(chunk(b"MOGP", &mogp));
    file
}

/// A WMO placed at the origin whose group holds `faces` in one BSP leaf.
fn single_leaf_wmo(faces: &[Face]) -> WmoCollision {
    wmo_at(Affine3A::IDENTITY, faces)
}

fn wmo_at(world_from_local: Affine3A, faces: &[Face]) -> WmoCollision {
    let file = group_file(0, faces, &[leaf(0..faces.len() as u32)]);
    let group = WmoGroupCollision::parse(&file).expect("synthetic group parses");
    WmoCollision::new(world_from_local, vec![Arc::new(group)])
}

/// Ground under `feet`, height rounded to 1e-4 yd (segment hits are f32 lerps).
fn ground_height(wmo: &WmoCollision, terrain: Option<f32>, feet: Vec3) -> Option<(f32, Surface)> {
    ground_at(feet, terrain, [wmo])
        .map(|ground| ((ground.height * 1e4).round() / 1e4, ground.surface))
}

#[test]
fn ground_is_the_highest_floor_within_step_reach() {
    let faces: Vec<Face> = floor(0.0).into_iter().chain(floor(3.0)).collect();
    let wmo = single_leaf_wmo(&faces);

    assert_eq!(
        ground_height(&wmo, None, Vec3::new(1.0, 3.2, 1.0)),
        Some((3.0, Surface::Wmo))
    );
    assert_eq!(
        ground_height(&wmo, None, Vec3::new(1.0, 0.4, 1.0)),
        Some((0.0, Surface::Wmo)),
        "the upper floor 2.6 above the feet is a ceiling"
    );
    assert_eq!(
        ground_height(&wmo, None, Vec3::new(1.0, 1.5, 1.0)),
        Some((3.0, Surface::Wmo)),
        "a floor 1.5 above the feet is within step reach"
    );
}

#[test]
fn wmo_floor_over_terrain_supports_until_the_character_is_below_it() {
    let faces = floor(98.06);
    let wmo = single_leaf_wmo(&faces);

    assert_eq!(
        ground_height(&wmo, Some(94.7), Vec3::new(0.0, 98.06, 0.0)),
        Some((98.06, Surface::Wmo))
    );
    assert_eq!(
        ground_height(&wmo, Some(94.7), Vec3::new(0.0, 94.7, 0.0)),
        Some((94.7, Surface::Terrain)),
        "3.36 below the floor the terrain is the ground"
    );
    assert_eq!(
        ground_height(&wmo, Some(94.7), Vec3::new(30.0, 98.06, 0.0)),
        Some((94.7, Surface::Terrain)),
        "outside the floor's footprint only the terrain supports"
    );
}

#[test]
fn only_collidable_walkable_faces_support() {
    let detail = quad([-10.0, 10.0], [-10.0, 10.0], 2.0, RENDER | DETAIL, 0);
    let collision_only = quad([-10.0, 10.0], [-10.0, 10.0], 1.0, 0, 0xFF);
    let unflagged = quad([-10.0, 10.0], [-10.0, 10.0], 1.5, 0, 3);
    let faces: Vec<Face> = detail
        .into_iter()
        .chain(collision_only)
        .chain(unflagged)
        .collect();
    let wmo = single_leaf_wmo(&faces);

    assert_eq!(
        ground_height(&wmo, None, Vec3::new(0.0, 2.0, 0.0)),
        Some((1.0, Surface::Wmo)),
        "render-only detail and unflagged faces do not collide; collision-only faces do"
    );
}

#[test]
fn slopes_steeper_than_the_walkable_limit_do_not_support() {
    // 60° ramp rising along file X, and a 30° ramp beside it along file Y.
    let steep = Face {
        corners: [
            [-1.0, -1.0, 0.0],
            [1.0, -1.0, 2.0 * 3f32.sqrt()],
            [1.0, 1.0, 2.0 * 3f32.sqrt()],
        ],
        flags: RENDER,
        material: 0,
    };
    let gentle_rise = 20.0 / 3f32.sqrt();
    let gentle = Face {
        corners: [
            [20.0, 0.0, 0.0],
            [40.0, 0.0, 0.0],
            [40.0, 20.0, gentle_rise],
        ],
        flags: RENDER,
        material: 0,
    };
    let wmo = single_leaf_wmo(&[steep, gentle]);

    assert_eq!(ground_height(&wmo, None, Vec3::new(0.5, 3.0, 0.0)), None);
    assert_eq!(
        ground_height(&wmo, None, Vec3::new(35.0, 3.0, -5.0)),
        Some((2.8868, Surface::Wmo)),
        "the 30° ramp is 5 / √3 high 5 yd up its slope"
    );
}

#[test]
fn bsp_split_on_the_y_axis_routes_each_side_to_its_leaf() {
    let south = quad([-10.0, 10.0], [-10.0, 0.0], 1.0, RENDER, 0);
    let north = quad([-10.0, 10.0], [0.0, 10.0], 2.0, RENDER, 0);
    let faces: Vec<Face> = south.into_iter().chain(north).collect();
    let nodes = [
        Node {
            flags: AXIS_Y,
            neg: 1,
            pos: 2,
            faces: 0..0,
            dist: 0.0,
        },
        leaf(0..2),
        leaf(2..4),
    ];
    let group = WmoGroupCollision::parse(&group_file(0, &faces, &nodes)).unwrap();
    let wmo = WmoCollision::new(Affine3A::IDENTITY, vec![Arc::new(group)]);

    // File Y is Bevy -Z.
    assert_eq!(
        ground_height(&wmo, None, Vec3::new(5.0, 2.5, 5.0)),
        Some((1.0, Surface::Wmo))
    );
    assert_eq!(
        ground_height(&wmo, None, Vec3::new(5.0, 2.5, -5.0)),
        Some((2.0, Surface::Wmo))
    );
}

#[test]
fn unreachable_and_antiportal_groups_have_no_floors() {
    let faces = floor(1.0);
    for flags in [0x80, 0x0400_0000] {
        let group = WmoGroupCollision::parse(&group_file(flags, &faces, &[leaf(0..2)])).unwrap();
        let wmo = WmoCollision::new(Affine3A::IDENTITY, vec![Arc::new(group)]);
        assert_eq!(ground_height(&wmo, None, Vec3::new(0.0, 1.0, 0.0)), None);
    }
}

#[test]
fn placed_wmo_floor_follows_its_placement_transform() {
    // Floor strip along file +X from 0 to 10, 4 wide.
    let faces = quad([0.0, 10.0], [-2.0, 2.0], 2.0, RENDER, 0);
    let placement = Affine3A::from_scale_rotation_translation(
        Vec3::ONE,
        Quat::from_rotation_y(std::f32::consts::FRAC_PI_2),
        Vec3::new(100.0, 50.0, 200.0),
    );
    let wmo = wmo_at(placement, &faces);

    // Local +X turns to world -Z under a 90° turn about Y.
    assert_eq!(
        ground_height(&wmo, None, Vec3::new(100.0, 52.0, 195.0)),
        Some((52.0, Surface::Wmo))
    );
    assert_eq!(
        ground_height(&wmo, None, Vec3::new(105.0, 52.0, 200.0)),
        None
    );
}

#[test]
fn placement_position_and_rotation_follow_the_adt_convention() {
    let center = 32.0 * crate::navmesh::TILE_SIZE;
    assert_eq!(
        placement_position([center + 10.0, 5.0, center - 20.0]),
        Vec3::new(20.0, 5.0, 10.0)
    );
    // Stored heading 180° is no turn in Bevy.
    let unturned = placement_rotation([0.0, 180.0, 0.0]) * Vec3::X;
    assert!(unturned.abs_diff_eq(Vec3::X, 1e-6), "{unturned}");
}

#[test]
fn global_wmo_placement_puts_file_coordinates_at_their_world_position() {
    // Stormwind Stockade (map 34): WDT 791060 places WMO 108631 at raw (0, 0, 0),
    // rotation 0. A floor under the entrance world_safe_locs 3599, WoW
    // (56.68, 0.62, -19.27), sits at file (-56.68, -0.62) (WoW X and Y turned 180°).
    let faces = quad([-60.0, -53.0], [-3.0, 2.0], -20.0, RENDER, 0);
    let placement = Affine3A::from_rotation_translation(
        placement_rotation([0.0, 0.0, 0.0]),
        global_wmo_placement_position([0.0, 0.0, 0.0]),
    );
    let wmo = wmo_at(placement, &faces);

    // WoW (56.68, 0.62, -19.5) is Bevy (56.68, -19.5, -0.62).
    assert_eq!(
        ground_height(&wmo, None, Vec3::new(56.68, -19.5, -0.62)),
        Some((-20.0, Surface::Wmo))
    );
    assert_eq!(
        ground_height(&wmo, None, Vec3::new(-56.68, -19.5, 0.62)),
        None
    );
}

#[test]
fn terrain_alone_supports_within_reach_and_a_higher_terrain_does_not() {
    assert_eq!(
        select_ground(10.0, Some(10.5), []),
        Some(Ground {
            height: 10.5,
            surface: Surface::Terrain
        })
    );
    assert_eq!(select_ground(10.0, Some(12.0), []), None);
}

/// Walls are solid too: every collidable face, floor or not, in WMO-local Bevy space
/// (`[x, z, -y]` of the file). Render detail and unflagged faces, and antiportal groups, have none.
#[test]
fn collidable_triangles_are_the_solid_faces_in_local_bevy_space() {
    let wall = Face {
        corners: [[0.0, 3.0, 0.0], [4.0, 3.0, 0.0], [4.0, 3.0, 5.0]],
        flags: RENDER,
        material: 0,
    };
    let collision_only = Face {
        corners: [[0.0, 0.0, 1.0], [1.0, 0.0, 1.0], [1.0, 1.0, 1.0]],
        flags: 0,
        material: 0xFF,
    };
    let detail = Face {
        corners: [[0.0, 0.0, 2.0], [1.0, 0.0, 2.0], [1.0, 1.0, 2.0]],
        flags: RENDER | DETAIL,
        material: 0,
    };
    let unflagged = Face {
        corners: [[0.0, 0.0, 3.0], [1.0, 0.0, 3.0], [1.0, 1.0, 3.0]],
        flags: 0,
        material: 3,
    };
    let faces = [wall, collision_only, detail, unflagged];
    let group = WmoGroupCollision::parse(&group_file(0, &faces, &[leaf(0..4)])).unwrap();

    assert_eq!(
        group.collidable_triangles().collect::<Vec<_>>(),
        vec![
            [
                Vec3::new(0.0, 0.0, -3.0),
                Vec3::new(4.0, 0.0, -3.0),
                Vec3::new(4.0, 5.0, -3.0)
            ],
            [
                Vec3::new(0.0, 1.0, 0.0),
                Vec3::new(1.0, 1.0, 0.0),
                Vec3::new(1.0, 1.0, -1.0)
            ],
        ]
    );
    assert_eq!(
        group.local_bounds(),
        (Vec3::new(0.0, 0.0, -3.0), Vec3::new(4.0, 5.0, 0.0)),
        "file bbox (0,0,0)..(4,3,5) in local Bevy space"
    );
    let antiportal =
        WmoGroupCollision::parse(&group_file(0x0400_0000, &faces, &[leaf(0..4)])).unwrap();
    assert_eq!(antiportal.collidable_triangles().count(), 0);
}
