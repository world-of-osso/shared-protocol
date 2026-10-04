//! WMO group collision: the MOPY-collidable faces reached through the group's
//! MOBN/MOBR BSP, in the file's WMO-local coordinates (X, Y, Z up).

use bevy::math::{Affine3A, Vec3};

/// MOGP header bytes before the group's sub-chunks.
const MOGP_HEADER_SIZE: usize = 0x44;
/// MOGP flag: the group cannot be entered (AzerothCore `WMOGroup::ShouldSkip`).
const MOGP_UNREACHABLE: u32 = 0x80;
/// MOGP flag: antiportal occluder, not geometry.
const MOGP_ANTIPORTAL: u32 = 0x0400_0000;

const MOPY_DETAIL: u16 = 0x04;
const MOPY_COLLISION: u16 = 0x08;
const MOPY_RENDER: u16 = 0x20;
/// MOPY material id of a collision-only face.
const MOPY_COLLISION_ONLY_MATERIAL: u8 = 0xFF;

const MOBN_AXIS_MASK: u16 = 0x03;
const MOBN_LEAF: u16 = 0x04;
const MOBN_ENTRY_SIZE: usize = 16;

/// Parallel-ray rejection for the segment/triangle test.
const RAY_EPSILON: f32 = 1e-7;

struct BspNode {
    flags: u16,
    neg_child: i16,
    pos_child: i16,
    face_count: u16,
    face_start: u32,
    plane_dist: f32,
}

#[derive(Default)]
pub struct WmoGroupCollision {
    bbox_min: Vec3,
    bbox_max: Vec3,
    vertices: Vec<Vec3>,
    indices: Vec<u16>,
    /// Per triangle: whether MOPY makes it collide.
    collidable: Vec<bool>,
    nodes: Vec<BspNode>,
    face_refs: Vec<u16>,
}

impl WmoGroupCollision {
    /// Parse a WMO group file (MVER + MOGP). Unreachable and antiportal
    /// groups parse to a group without faces.
    pub fn parse(group_file: &[u8]) -> Result<Self, String> {
        let mogp = find_chunk(group_file, b"MOGP")?.ok_or("WMO group has no MOGP chunk")?;
        if mogp.len() < MOGP_HEADER_SIZE {
            return Err(format!("MOGP payload too small: {} bytes", mogp.len()));
        }
        let flags = read_u32(mogp, 8)?;
        let mut group = Self {
            bbox_min: read_vec3(mogp, 12)?,
            bbox_max: read_vec3(mogp, 24)?,
            vertices: Vec::new(),
            indices: Vec::new(),
            collidable: Vec::new(),
            nodes: Vec::new(),
            face_refs: Vec::new(),
        };
        if flags & (MOGP_UNREACHABLE | MOGP_ANTIPORTAL) != 0 {
            return Ok(group);
        }
        for chunk in Chunks(&mogp[MOGP_HEADER_SIZE..]) {
            let (tag, payload) = chunk?;
            match &tag {
                b"MOPY" => group.collidable = parse_mopy(payload),
                b"MPY2" => group.collidable = parse_mpy2(payload),
                b"MOVI" => group.indices = parse_u16s(payload),
                b"MOVT" => group.vertices = parse_vec3s(payload)?,
                b"MOBN" => group.nodes = parse_mobn(payload)?,
                b"MOBR" => group.face_refs = parse_u16s(payload),
                _ => {}
            }
        }
        Ok(group)
    }

    /// Every MOPY-collidable triangle in WMO-local Bevy space: the solid
    /// surface of the group, walls and floors alike. Unreachable and
    /// antiportal groups have none.
    pub fn collidable_triangles(&self) -> impl Iterator<Item = [Vec3; 3]> + '_ {
        let faces = self.indices.len() / 3;
        (0..faces)
            .filter_map(|face| u16::try_from(face).ok())
            .filter(|&face| self.face_collides(face))
            .filter_map(|face| self.face_vertices(face))
            .map(|corners| corners.map(file_to_local_bevy))
    }

    /// Push the world height of every walkable face the local segment
    /// `top..bottom` (WMO-local Bevy space) crosses.
    pub(super) fn walkable_hits(
        &self,
        top: Vec3,
        bottom: Vec3,
        world_from_local: &Affine3A,
        out: &mut Vec<f32>,
    ) {
        if self.nodes.is_empty() {
            return;
        }
        let (start, end) = (local_bevy_to_file(top), local_bevy_to_file(bottom));
        if !self.bbox_overlaps(start, end) {
            return;
        }
        let min_normal_y = super::min_walkable_normal_y();
        self.for_each_leaf_face(start, end, |face| {
            let Some([a, b, c]) = self.face_vertices(face) else {
                return;
            };
            let Some(t) = segment_hits_triangle(start, end, [a, b, c]) else {
                return;
            };
            let normal = (b - a).cross(c - a);
            let world_normal = world_from_local
                .matrix3
                .mul_vec3(file_to_local_bevy(normal).into())
                .normalize_or_zero();
            if world_normal.y.abs() >= min_normal_y {
                let hit = file_to_local_bevy(start.lerp(end, t));
                out.push(world_from_local.transform_point3(hit).y);
            }
        });
    }

    /// The MOGP bounding box in WMO-local Bevy space, as `(min, max)`.
    pub fn local_bounds(&self) -> (Vec3, Vec3) {
        let (a, b) = (
            file_to_local_bevy(self.bbox_min),
            file_to_local_bevy(self.bbox_max),
        );
        (a.min(b), a.max(b))
    }

    /// Corners of the MOGP bounding box in WMO-local Bevy space.
    pub(super) fn local_corners(&self) -> impl Iterator<Item = Vec3> + '_ {
        (0..8).map(|bits: u8| {
            let pick = |bit: u8, axis: usize| {
                if bits & bit == 0 {
                    self.bbox_min[axis]
                } else {
                    self.bbox_max[axis]
                }
            };
            file_to_local_bevy(Vec3::new(pick(1, 0), pick(2, 1), pick(4, 2)))
        })
    }

    fn bbox_overlaps(&self, start: Vec3, end: Vec3) -> bool {
        let min = start.min(end);
        let max = start.max(end);
        min.cmple(self.bbox_max).all() && max.cmpge(self.bbox_min).all()
    }

    /// Visit the faces of every BSP leaf the segment reaches.
    fn for_each_leaf_face(&self, start: Vec3, end: Vec3, mut visit: impl FnMut(u16)) {
        let (min, max) = (start.min(end), start.max(end));
        let mut stack = vec![0usize];
        while let Some(index) = stack.pop() {
            let Some(node) = self.nodes.get(index) else {
                continue;
            };
            if node.flags & MOBN_LEAF != 0 {
                let first = node.face_start as usize;
                let last = (first + node.face_count as usize).min(self.face_refs.len());
                for &face in self.face_refs.get(first..last).unwrap_or_default() {
                    if self.face_collides(face) {
                        visit(face);
                    }
                }
                continue;
            }
            let axis = usize::from(node.flags & MOBN_AXIS_MASK).min(2);
            if min[axis] <= node.plane_dist && node.neg_child >= 0 {
                stack.push(node.neg_child as usize);
            }
            if max[axis] >= node.plane_dist && node.pos_child >= 0 {
                stack.push(node.pos_child as usize);
            }
        }
    }

    fn face_collides(&self, face: u16) -> bool {
        self.collidable
            .get(usize::from(face))
            .copied()
            .unwrap_or(false)
    }

    fn face_vertices(&self, face: u16) -> Option<[Vec3; 3]> {
        let base = usize::from(face) * 3;
        let corner = |offset: usize| {
            let index = *self.indices.get(base + offset)?;
            self.vertices.get(usize::from(index)).copied()
        };
        Some([corner(0)?, corner(1)?, corner(2)?])
    }
}

/// AzerothCore vmap4_extractor `wmo.cpp`: a face collides when flagged
/// COLLISION, when it renders and is not DETAIL, or when its material is the
/// collision-only id.
fn face_collides(flags: u16, collision_only: bool) -> bool {
    let render_face = flags & MOPY_RENDER != 0 && flags & MOPY_DETAIL == 0;
    flags & MOPY_COLLISION != 0 || render_face || collision_only
}

/// Two-sided Möller–Trumbore; returns the segment parameter of the hit.
fn segment_hits_triangle(start: Vec3, end: Vec3, [a, b, c]: [Vec3; 3]) -> Option<f32> {
    let dir = end - start;
    let edge1 = b - a;
    let edge2 = c - a;
    let p = dir.cross(edge2);
    let det = edge1.dot(p);
    if det.abs() < RAY_EPSILON {
        return None;
    }
    let inv_det = 1.0 / det;
    let s = start - a;
    let u = s.dot(p) * inv_det;
    if !(0.0..=1.0).contains(&u) {
        return None;
    }
    let q = s.cross(edge1);
    let v = dir.dot(q) * inv_det;
    if v < 0.0 || u + v > 1.0 {
        return None;
    }
    let t = edge2.dot(q) * inv_det;
    (0.0..=1.0).contains(&t).then_some(t)
}

/// WMO-local Bevy `[x, z, -y]` back to the file's `[x, y, z]`.
fn local_bevy_to_file(point: Vec3) -> Vec3 {
    Vec3::new(point.x, -point.z, point.y)
}

fn file_to_local_bevy(point: Vec3) -> Vec3 {
    Vec3::new(point.x, point.z, -point.y)
}

fn parse_mopy(payload: &[u8]) -> Vec<bool> {
    payload
        .chunks_exact(2)
        .map(|entry| {
            face_collides(
                u16::from(entry[0]),
                entry[1] == MOPY_COLLISION_ONLY_MATERIAL,
            )
        })
        .collect()
}

/// Forever MPY2 widens MOPY flags and material indices to little-endian u16s.
fn parse_mpy2(payload: &[u8]) -> Vec<bool> {
    payload
        .chunks_exact(4)
        .map(|entry| {
            let flags = u16::from_le_bytes([entry[0], entry[1]]);
            let material = u16::from_le_bytes([entry[2], entry[3]]);
            face_collides(flags, material == u16::MAX)
        })
        .collect()
}

fn parse_mobn(payload: &[u8]) -> Result<Vec<BspNode>, String> {
    payload
        .chunks_exact(MOBN_ENTRY_SIZE)
        .map(|entry| {
            Ok(BspNode {
                flags: read_u16(entry, 0)?,
                neg_child: read_u16(entry, 2)? as i16,
                pos_child: read_u16(entry, 4)? as i16,
                face_count: read_u16(entry, 6)?,
                face_start: read_u32(entry, 8)?,
                plane_dist: f32::from_bits(read_u32(entry, 12)?),
            })
        })
        .collect()
}

fn parse_u16s(payload: &[u8]) -> Vec<u16> {
    payload
        .chunks_exact(2)
        .map(|bytes| u16::from_le_bytes([bytes[0], bytes[1]]))
        .collect()
}

fn parse_vec3s(payload: &[u8]) -> Result<Vec<Vec3>, String> {
    (0..payload.len() / 12)
        .map(|index| read_vec3(payload, index * 12))
        .collect()
}

/// Payload of the first chunk tagged `tag` (tags are stored reversed).
fn find_chunk<'a>(data: &'a [u8], tag: &[u8; 4]) -> Result<Option<&'a [u8]>, String> {
    for chunk in Chunks(data) {
        let (chunk_tag, payload) = chunk?;
        if &chunk_tag == tag {
            return Ok(Some(payload));
        }
    }
    Ok(None)
}

/// IFF chunk iterator yielding `(tag, payload)` with the tag un-reversed.
struct Chunks<'a>(&'a [u8]);

impl<'a> Iterator for Chunks<'a> {
    type Item = Result<([u8; 4], &'a [u8]), String>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.0.len() < 8 {
            return None;
        }
        let tag = [self.0[3], self.0[2], self.0[1], self.0[0]];
        let size = u32::from_le_bytes([self.0[4], self.0[5], self.0[6], self.0[7]]) as usize;
        let Some(payload) = self.0.get(8..8 + size) else {
            let tag = String::from_utf8_lossy(&tag).into_owned();
            self.0 = &[];
            return Some(Err(format!("{tag} chunk truncated")));
        };
        self.0 = &self.0[8 + size..];
        Some(Ok((tag, payload)))
    }
}

fn read_u16(data: &[u8], offset: usize) -> Result<u16, String> {
    data.get(offset..offset + 2)
        .map(|bytes| u16::from_le_bytes([bytes[0], bytes[1]]))
        .ok_or_else(|| format!("read_u16 out of bounds at {offset:#x}"))
}

fn read_u32(data: &[u8], offset: usize) -> Result<u32, String> {
    data.get(offset..offset + 4)
        .map(|bytes| u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
        .ok_or_else(|| format!("read_u32 out of bounds at {offset:#x}"))
}

fn read_vec3(data: &[u8], offset: usize) -> Result<Vec3, String> {
    Ok(Vec3::new(
        f32::from_bits(read_u32(data, offset)?),
        f32::from_bits(read_u32(data, offset + 4)?),
        f32::from_bits(read_u32(data, offset + 8)?),
    ))
}
