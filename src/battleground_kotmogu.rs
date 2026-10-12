//! Kotmogu orb authority: TC a352b1fa battleground_temple_of_kotmogu.cpp;
//! TDB1210 areatrigger spawns 100/101, spell_area 112054/6051, orb templates.
use std::collections::BTreeMap;

use super::{BgInstance, BgOutcome, BgPhase, BgTeam, BgTemplate};

pub const KOTMOGU_STARTS: [u32; 2] = [4059, 4060];
pub const ORB_POSSESSIONS_STAT: u32 = 418;
pub const ORB_VICTORY_POINTS_STAT: u32 = 419;
const POINT_TICK_MS: u64 = 5000;
const AREA_CENTER: [f32; 3] = [1783.41, 1333.56, 6.0];
const SMALL_EXTENTS: [f32; 3] = [33.0, 47.0, 2.5];
const MEDIUM_RADIUS: f32 = 50.0;

#[derive(Debug)]
pub struct KotmoguOrb {
    pub entry: u32,
    pub name: &'static str,
    pub position: [f32; 3],
    /// In-base, Horde-held, Alliance-held WorldState IDs from the pinned script.
    pub world_states: [u32; 3],
}

pub const KOTMOGU_ORBS: [KotmoguOrb; 4] = [
    KotmoguOrb {
        entry: 212094,
        name: "Orange Orb",
        position: [1850.1666, 1250.118, 13.2056],
        world_states: [6717, 6963, 6964],
    },
    KotmoguOrb {
        entry: 212091,
        name: "Blue Orb",
        position: [1716.9479, 1250.0173, 13.3266],
        world_states: [6714, 6969, 6970],
    },
    KotmoguOrb {
        entry: 212093,
        name: "Green Orb",
        position: [1716.8923, 1416.618, 13.2056],
        world_states: [6716, 6965, 6966],
    },
    KotmoguOrb {
        entry: 212092,
        name: "Purple Orb",
        position: [1850.217, 1416.8229, 13.3382],
        world_states: [6715, 6967, 6968],
    },
];

#[derive(Debug)]
pub struct KotmoguBattle {
    carriers: [Option<(u64, BgTeam)>; 4],
    tick_left_ms: u64,
}

impl Default for KotmoguBattle {
    fn default() -> Self {
        Self::new()
    }
}

impl KotmoguBattle {
    pub fn new() -> Self {
        Self {
            carriers: [None; 4],
            tick_left_ms: POINT_TICK_MS,
        }
    }

    pub fn pickup(&mut self, entry: u32, player: u64, team: BgTeam) -> Result<(), String> {
        if self.carriers.iter().flatten().any(|(id, _)| *id == player) {
            return Err("player already carries an Orb of Power".into());
        }
        let index = KOTMOGU_ORBS
            .iter()
            .position(|orb| orb.entry == entry)
            .ok_or("not a Kotmogu Orb of Power")?;
        if self.carriers[index].is_some() {
            return Err("Orb of Power is already carried".into());
        }
        self.carriers[index] = Some((player, team));
        Ok(())
    }

    pub fn drop_carrier(&mut self, player: u64) -> bool {
        let carrier = self
            .carriers
            .iter_mut()
            .find(|carrier| carrier.is_some_and(|(id, _)| id == player));
        let Some(carrier) = carrier else { return false };
        *carrier = None;
        true
    }

    pub fn holders(&self) -> Vec<(u32, u64, BgTeam)> {
        KOTMOGU_ORBS
            .iter()
            .zip(self.carriers)
            .filter_map(|(orb, carrier)| carrier.map(|(id, team)| (orb.entry, id, team)))
            .collect()
    }

    /// `players` contains only alive, arrived participants on the isolated match map.
    /// Death or departure returns the orb immediately, never as a dropped ground object.
    pub fn advance(
        &mut self,
        mut delta_ms: u64,
        players: &[(u64, [f32; 3])],
        instance: &mut BgInstance,
        template: &BgTemplate,
    ) -> Vec<(u64, u32)> {
        if instance.phase != BgPhase::InProgress {
            return Vec::new();
        }
        self.return_absent_carriers(players);
        let mut credited = BTreeMap::new();
        while delta_ms > 0 && instance.phase == BgPhase::InProgress {
            let step = delta_ms.min(self.tick_left_ms);
            delta_ms -= step;
            self.tick_left_ms -= step;
            if self.tick_left_ms == 0 {
                self.score_tick(players, instance, template, &mut credited);
                self.tick_left_ms = POINT_TICK_MS;
            }
        }
        credited.into_iter().collect()
    }

    fn return_absent_carriers(&mut self, players: &[(u64, [f32; 3])]) {
        for carrier in &mut self.carriers {
            if carrier.is_some_and(|(id, _)| !players.iter().any(|(player, _)| *player == id)) {
                *carrier = None;
            }
        }
    }

    fn score_tick(
        &self,
        players: &[(u64, [f32; 3])],
        instance: &mut BgInstance,
        template: &BgTemplate,
        credited: &mut BTreeMap<u64, u32>,
    ) {
        for (player, team) in self.carriers.iter().flatten() {
            let position = players
                .iter()
                .find(|(id, _)| id == player)
                .expect("absent carriers returned before tick")
                .1;
            let points = victory_points(position);
            *credited.entry(*player).or_default() += points;
            let score = match team {
                BgTeam::A => &mut instance.score_a.score,
                BgTeam::B => &mut instance.score_b.score,
            };
            *score = (*score + points).min(template.win_score);
        }
        // TC CheckWinner runs after every carrier has received the scheduled tick.
        // Do not use add_score, which would end on the first team and preclude a draw.
        finish_at_limit(instance, template.win_score);
    }

    pub fn world_states(&self, instance: &BgInstance) -> Vec<(u32, i32)> {
        let mut states = vec![
            (17388, 1500),
            (6303, instance.score_a.score as i32),
            (6304, instance.score_b.score as i32),
        ];
        for (orb, carrier) in KOTMOGU_ORBS.iter().zip(self.carriers) {
            let team = carrier.map(|(_, team)| team);
            states.extend([
                (orb.world_states[0], i32::from(team.is_none())),
                (orb.world_states[1], i32::from(team == Some(BgTeam::B))),
                (orb.world_states[2], i32::from(team == Some(BgTeam::A))),
            ]);
        }
        states
    }
}

/// TC AreaTrigger::SearchUnitInBox uses X/Y extents and half the Z extent.
/// The medium trigger's radius 50 and height 0 collapse G3D Cylinder::contains
/// to distance from its single endpoint (sphere), not an invented planar circle.
fn victory_points(position: [f32; 3]) -> u32 {
    let offset = std::array::from_fn::<_, 3, _>(|axis| position[axis] - AREA_CENTER[axis]);
    let inside_small = offset
        .iter()
        .zip(SMALL_EXTENTS)
        .all(|(distance, extent)| distance.abs() <= extent);
    if inside_small {
        return 6;
    }
    let distance_squared: f32 = offset.iter().map(|value| value * value).sum();
    if distance_squared <= MEDIUM_RADIUS * MEDIUM_RADIUS {
        return 4;
    }
    // TDB spell_area 112054 applies LargeAura throughout area 6051 (this battleground).
    2
}

fn finish_at_limit(instance: &mut BgInstance, limit: u32) {
    let outcome = match (
        instance.score_a.score == limit,
        instance.score_b.score == limit,
    ) {
        (true, true) => BgOutcome::Draw,
        (true, false) => BgOutcome::Winner(BgTeam::A),
        (false, true) => BgOutcome::Winner(BgTeam::B),
        (false, false) => return,
    };
    instance.phase = BgPhase::Ended;
    instance.outcome = Some(outcome);
}

#[cfg(test)]
#[path = "battleground_tests/kotmogu.rs"]
mod tests;
