//! Silvershard rules: TC a352b1fa battleground_silvershard_mines.cpp and
//! GameObject.cpp ControlZone; TDB1210 templates 213796/213794/213766 and paths.
use std::collections::BTreeMap;

use super::motion::{distance, follow_linear_path};
use super::{BgInstance, BgOutcome, BgPhase, BgTeam, BgTemplate};

pub type CartPaths = BTreeMap<u32, Vec<[f32; 3]>>;
pub const SILVERSHARD_STARTS: [u32; 2] = [4062, 4061];
pub const SILVERSHARD_CART_ENTRY: u32 = 60140;
pub const CARTS_CONTROLLED_STAT: u32 = 428;
const HEARTBEAT_MS: u64 = 1000;
const RESOURCE_TICK_MS: u64 = 2000;
const RESPAWN_MS: u64 = 5000;
const DELIVERY_POINTS: u32 = 150;
const CONTROL_RADIUS: f32 = 25.0;
// All three templates: neutralPercent 2, min/maxSuperiority 1, min/maxTime 50s.
const ZONE_START: i32 = 50;
const ZONE_RATE: i32 = 100 / 50;
const NEUTRAL_PERCENT: i32 = 2;
const ALLIANCE_ABOVE: i32 = ZONE_START + NEUTRAL_PERCENT / 2;
const HORDE_BELOW: i32 = ZONE_START - NEUTRAL_PERCENT / 2;
// TDB paths MoveType Walk, Velocity NULL; creature 60140 speed_walk 0.8.
// TC baseMoveSpeed[MOVE_WALK] 2.5; exact ground spline is linear, not smoothed.
const CART_SPEED: f32 = 2.5 * 0.8;

#[derive(Debug)]
pub struct CartDefinition {
    pub name: &'static str,
    pub spawn: [f32; 3],
    pub orientation: f32,
    pub control_entry: u32,
    pub initial_path: u32,
    pub world_states: [u32; 2],
    /// North has no neutral event in its source template; its displayed states
    /// retain the prior control until a faction event or depot reset updates them.
    neutral_event: bool,
}

pub const SILVERSHARD_CARTS: [CartDefinition; 3] = [
    CartDefinition {
        name: "South Cart",
        spawn: [739.29517, 203.76389, 319.54398],
        orientation: 2.2614408,
        control_entry: 213796,
        initial_path: 6014000,
        world_states: [6881, 6882],
        neutral_event: true,
    },
    CartDefinition {
        name: "North Cart",
        spawn: [759.32465, 198.3316, 319.53058],
        orientation: 0.42150828,
        control_entry: 213794,
        initial_path: 6014002,
        world_states: [6880, 6879],
        neutral_event: false,
    },
    CartDefinition {
        name: "East Cart",
        spawn: [744.5174, 183.19792, 319.54395],
        orientation: 4.338116,
        control_entry: 213766,
        initial_path: 6014007,
        world_states: [6439, 6440],
        neutral_event: true,
    },
];

#[derive(Debug, Clone, PartialEq)]
pub enum CartEvent {
    Controlled(u64),
    Delivered {
        cart: usize,
        team: Option<BgTeam>,
        position: [f32; 3],
    },
}

#[derive(Debug)]
struct MovingCart {
    position: [f32; 3],
    path: u32,
    next_node: usize,
    value: i32,
    displayed_owner: Option<BgTeam>,
    respawn_left_ms: Option<u64>,
}

impl MovingCart {
    fn new(definition: &CartDefinition) -> Self {
        Self {
            position: definition.spawn,
            path: definition.initial_path,
            next_node: 0,
            value: ZONE_START,
            displayed_owner: None,
            respawn_left_ms: None,
        }
    }

    fn owner(&self) -> Option<BgTeam> {
        if self.value > ALLIANCE_ABOVE {
            return Some(BgTeam::A);
        }
        if self.value < HORDE_BELOW {
            return Some(BgTeam::B);
        }
        None
    }

    fn advance_motion(&mut self, delta_ms: u64, paths: &CartPaths, switches: [i32; 2]) -> bool {
        let mut distance_left = delta_ms as f32 / 1000.0 * CART_SPEED;
        loop {
            let points = paths.get(&self.path).expect("validated source cart path");
            if !follow_linear_path(
                &mut self.position,
                &mut self.next_node,
                points,
                &mut distance_left,
            ) {
                return false;
            }
            let Some(next) = next_path(self.path, switches) else {
                return true;
            };
            self.path = next;
            self.next_node = 0;
        }
    }

    fn heartbeat(
        &mut self,
        definition: &CartDefinition,
        players: &[(u64, BgTeam, [f32; 3])],
    ) -> Vec<CartEvent> {
        let inside: Vec<_> = players
            .iter()
            .filter(|(_, _, at)| distance(*at, self.position) <= CONTROL_RADIUS)
            .collect();
        let a = inside
            .iter()
            .filter(|(_, team, _)| *team == BgTeam::A)
            .count();
        let b = inside
            .iter()
            .filter(|(_, team, _)| *team == BgTeam::B)
            .count();
        let direction = match a.cmp(&b) {
            std::cmp::Ordering::Greater => 1,
            std::cmp::Ordering::Less => -1,
            std::cmp::Ordering::Equal => 0,
        };
        let previous = self.owner();
        self.value = (self.value + direction * ZONE_RATE).clamp(0, 100);
        let owner = self.owner();
        if previous == owner {
            return Vec::new();
        }
        if owner.is_some() || definition.neutral_event {
            self.displayed_owner = owner;
        }
        inside
            .into_iter()
            .filter(|(_, team, _)| Some(*team) == owner)
            .map(|(id, _, _)| CartEvent::Controlled(*id))
            .collect()
    }
}

#[derive(Debug)]
pub struct SilvershardBattle {
    carts: [MovingCart; 3],
    /// Source WorldStates 6467 eastern default 2, 6468 northern default 1.
    switches: [i32; 2],
    heartbeat_left_ms: u64,
    tick_left_ms: u64,
    captured: [bool; 2],
}

impl Default for SilvershardBattle {
    fn default() -> Self {
        Self::new()
    }
}
impl SilvershardBattle {
    pub fn new() -> Self {
        Self {
            carts: std::array::from_fn(|index| MovingCart::new(&SILVERSHARD_CARTS[index])),
            switches: [2, 1],
            heartbeat_left_ms: HEARTBEAT_MS,
            tick_left_ms: RESOURCE_TICK_MS,
            captured: [false; 2],
        }
    }

    pub fn sites(&self) -> [Option<[f32; 3]>; 3] {
        std::array::from_fn(|index| {
            self.carts[index]
                .respawn_left_ms
                .is_none()
                .then_some(self.carts[index].position)
        })
    }

    pub fn switch_track(&mut self, state: u32) -> Result<(), String> {
        let index = match state {
            6467 => 0,
            6468 => 1,
            _ => return Err("not a Silvershard track switch".into()),
        };
        self.switches[index] = 3 - self.switches[index];
        Ok(())
    }

    pub fn world_states(&self, instance: &BgInstance) -> Vec<(u32, i32)> {
        let mut values = vec![
            (6437, instance.score_a.score as i32),
            (6438, instance.score_b.score as i32),
            (6467, self.switches[0]),
            (6468, self.switches[1]),
            (6954, i32::from(self.captured[0])),
            (6955, i32::from(self.captured[1])),
        ];
        for (cart, definition) in self.carts.iter().zip(&SILVERSHARD_CARTS) {
            values.extend([
                (
                    definition.world_states[0],
                    i32::from(cart.displayed_owner == Some(BgTeam::A)),
                ),
                (
                    definition.world_states[1],
                    i32::from(cart.displayed_owner == Some(BgTeam::B)),
                ),
            ]);
        }
        values
    }

    pub fn advance(
        &mut self,
        mut delta_ms: u64,
        players: &[(u64, BgTeam, [f32; 3])],
        paths: &CartPaths,
        instance: &mut BgInstance,
        template: &BgTemplate,
    ) -> Vec<CartEvent> {
        let mut events = Vec::new();
        while delta_ms > 0 && instance.phase == BgPhase::InProgress {
            let step = delta_ms.min(self.heartbeat_left_ms).min(self.tick_left_ms);
            delta_ms -= step;
            self.heartbeat_left_ms -= step;
            self.tick_left_ms -= step;
            self.advance_carts(step, paths, instance, template, &mut events);
            if instance.phase != BgPhase::InProgress {
                break;
            }
            if self.heartbeat_left_ms == 0 {
                events.extend(self.heartbeat(players));
                self.heartbeat_left_ms = HEARTBEAT_MS;
            }
            if self.tick_left_ms == 0 {
                self.score_tick(instance, template);
                self.tick_left_ms = RESOURCE_TICK_MS;
            }
        }
        events
    }

    fn heartbeat(&mut self, players: &[(u64, BgTeam, [f32; 3])]) -> Vec<CartEvent> {
        self.carts
            .iter_mut()
            .zip(&SILVERSHARD_CARTS)
            .filter(|(cart, _)| cart.respawn_left_ms.is_none())
            .flat_map(|(cart, definition)| cart.heartbeat(definition, players))
            .collect()
    }

    fn advance_carts(
        &mut self,
        step: u64,
        paths: &CartPaths,
        instance: &mut BgInstance,
        template: &BgTemplate,
        events: &mut Vec<CartEvent>,
    ) {
        for (index, cart) in self.carts.iter_mut().enumerate() {
            if let Some(left) = cart.respawn_left_ms {
                if left <= step {
                    *cart = MovingCart::new(&SILVERSHARD_CARTS[index]);
                } else {
                    cart.respawn_left_ms = Some(left - step);
                }
                continue;
            }
            if !cart.advance_motion(step, paths, self.switches) {
                continue;
            }
            let team = cart.owner();
            events.push(CartEvent::Delivered {
                cart: index,
                team,
                position: cart.position,
            });
            cart.respawn_left_ms = Some(RESPAWN_MS);
            cart.displayed_owner = None;
            if let Some(team) = team.filter(|_| instance.phase == BgPhase::InProgress) {
                self.captured[if team == BgTeam::A { 0 } else { 1 }] = true;
                instance.add_score(team, DELIVERY_POINTS, template);
            }
        }
    }

    fn score_tick(&self, instance: &mut BgInstance, template: &BgTemplate) {
        let count = |team| {
            self.carts
                .iter()
                .filter(|cart| cart.displayed_owner == Some(team))
                .count() as u32
        };
        instance.score_a.score =
            (instance.score_a.score + count(BgTeam::A)).min(template.win_score);
        instance.score_b.score =
            (instance.score_b.score + count(BgTeam::B)).min(template.win_score);
        let outcome = match (
            instance.score_a.score == template.win_score,
            instance.score_b.score == template.win_score,
        ) {
            (true, true) => BgOutcome::Draw,
            (true, false) => BgOutcome::Winner(BgTeam::A),
            (false, true) => BgOutcome::Winner(BgTeam::B),
            (false, false) => return,
        };
        instance.phase = BgPhase::Ended;
        instance.outcome = Some(outcome);
    }
}

fn next_path(path: u32, switches: [i32; 2]) -> Option<u32> {
    match path {
        6014000 => Some(6014001),
        6014002 => Some(if switches[1] == 1 { 6014005 } else { 6014003 }),
        6014003 => Some(6014004),
        6014005 => Some(6014006),
        6014007 => Some(if switches[0] == 1 { 6014010 } else { 6014008 }),
        6014008 => Some(6014009),
        6014010 => Some(6014011),
        _ => None,
    }
}

#[cfg(test)]
#[path = "battleground_tests/silvershard.rs"]
mod tests;
