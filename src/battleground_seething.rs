//! Seething Shore: pinned TC a352b1fa script, local Retail69933 SpellDuration64,
//! summon spell effects262749/248674/262748 and TDB WorldState13846.
use std::collections::BTreeMap;

use super::{BgInstance, BgPhase, BgTeam, BgTemplate};

pub const SEETHING_AREAS: [u32; 12] = [
    9516, 9521, 9518, 9511, 9513, 9519, 9517, 9520, 9515, 9522, 9514, 9512,
];
pub const SEETHING_OBJECTS: [u32; 3] = [281307, 272471, 281306];
pub const SEETHING_STARTS: [u32; 2] = [6337, 6380];
pub const AZERITE_COLLECTED_STAT: u32 = 744;
pub const AZERITE_GATHERING_SPELL: u32 = 261857;
const INITIAL_AREAS: [u32; 3] = [9516, 9515, 9521];
const MAX_ACTIVE: usize = 3;
const SPAWN_CHECK_MS: u64 = 2000;
const FILL_CHECK_MS: u64 = 100;
const GEYSER_MS: u64 = 40_000;
const DESPAWN_MS: u64 = 2000;
const CAPTURE_POINTS: u32 = 100;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AzeriteEvent {
    Activated {
        area: u32,
        generation: u64,
    },
    Spawned {
        area: u32,
        generation: u64,
        entry: u32,
    },
    Removed {
        generation: u64,
    },
}

#[derive(Debug)]
struct AzeriteNode {
    generation: u64,
    ready_at: u64,
    entry: Option<u32>,
}

/// Random selection is supplied by the server; pure tests pass concrete choices.
/// Stage-local clocks start at zero; this does not model Retail preparation stages.
#[derive(Debug)]
pub struct SeethingBattle {
    now: u64,
    next_check: u64,
    next_generation: u64,
    initial: bool,
    active: BTreeMap<u32, AzeriteNode>,
    captured: Vec<(u64, u64)>,
}

impl Default for SeethingBattle {
    fn default() -> Self {
        Self::new()
    }
}

impl SeethingBattle {
    pub fn new() -> Self {
        Self {
            now: 0,
            next_check: SPAWN_CHECK_MS,
            next_generation: 1,
            initial: true,
            active: BTreeMap::new(),
            captured: Vec::new(),
        }
    }

    pub fn advance(
        &mut self,
        delta_ms: u64,
        choose: &mut impl FnMut(usize) -> usize,
    ) -> Vec<AzeriteEvent> {
        let end = self.now + delta_ms;
        let mut events = Vec::new();
        while let Some(deadline) = self.next_deadline().filter(|deadline| *deadline <= end) {
            self.now = deadline;
            self.spawn_ready_objects(choose, &mut events);
            self.remove_captured_objects(&mut events);
            if self.next_check == deadline {
                self.refill_nodes(choose, &mut events);
            }
        }
        self.now = end;
        events
    }

    fn next_deadline(&self) -> Option<u64> {
        let geysers = self
            .active
            .values()
            .filter(|node| node.entry.is_none())
            .map(|node| node.ready_at);
        let captures = self.captured.iter().map(|(_, deadline)| *deadline);
        geysers.chain(captures).chain([self.next_check]).min()
    }

    fn spawn_ready_objects(
        &mut self,
        choose: &mut impl FnMut(usize) -> usize,
        events: &mut Vec<AzeriteEvent>,
    ) {
        for (&area, node) in &mut self.active {
            if node.entry.is_none() && node.ready_at == self.now {
                let entry = SEETHING_OBJECTS[choose(SEETHING_OBJECTS.len())];
                node.entry = Some(entry);
                events.push(AzeriteEvent::Spawned {
                    area,
                    generation: node.generation,
                    entry,
                });
            }
        }
    }

    fn remove_captured_objects(&mut self, events: &mut Vec<AzeriteEvent>) {
        self.captured.retain(|(generation, deadline)| {
            if *deadline == self.now {
                events.push(AzeriteEvent::Removed {
                    generation: *generation,
                });
                false
            } else {
                true
            }
        });
    }

    fn refill_nodes(
        &mut self,
        choose: &mut impl FnMut(usize) -> usize,
        events: &mut Vec<AzeriteEvent>,
    ) {
        let selected = if self.initial {
            self.initial = false;
            INITIAL_AREAS.to_vec()
        } else {
            self.select_inactive_areas(choose)
        };
        for area in selected {
            let generation = self.next_generation;
            self.next_generation += 1;
            self.active.insert(
                area,
                AzeriteNode {
                    generation,
                    ready_at: self.now + GEYSER_MS,
                    entry: None,
                },
            );
            events.push(AzeriteEvent::Activated { area, generation });
        }
        self.next_check = self.now
            + if self.active.len() < MAX_ACTIVE {
                FILL_CHECK_MS
            } else {
                SPAWN_CHECK_MS
            };
    }

    fn select_inactive_areas(&self, choose: &mut impl FnMut(usize) -> usize) -> Vec<u32> {
        let mut available: Vec<_> = SEETHING_AREAS
            .into_iter()
            .filter(|area| !self.active.contains_key(area))
            .collect();
        let count = MAX_ACTIVE - self.active.len();
        (0..count)
            .map(|_| {
                let index = choose(available.len());
                available.swap_remove(index)
            })
            .collect()
    }

    pub fn capture(
        &mut self,
        generation: u64,
        team: BgTeam,
        instance: &mut BgInstance,
        template: &BgTemplate,
    ) -> Result<(), String> {
        if instance.phase != BgPhase::InProgress {
            return Err("Azerite match is not active".into());
        }
        let area = self
            .active
            .iter()
            .find_map(|(&area, node)| {
                (node.generation == generation && node.entry.is_some()).then_some(area)
            })
            .ok_or("Azerite is not available")?;
        self.active.remove(&area);
        self.captured.push((generation, self.now + DESPAWN_MS));
        instance.add_score(team, CAPTURE_POINTS, template);
        Ok(())
    }

    pub fn world_states(&self, instance: &BgInstance) -> Vec<(u32, i32)> {
        vec![
            (13845, instance.score_a.score as i32),
            (13844, instance.score_b.score as i32),
            (13846, 1500),
        ]
    }
}
