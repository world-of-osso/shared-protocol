//! Retail capture-the-flag maps: TC a352b1fa battleground_warsong_gulch.cpp and
//! battleground_twin_peaks.cpp share flag entries, capture triggers30/31 and rules.
use super::{BgInstance, BgOutcome, BgPhase, BgTeam, BgTemplate, FlagObjective, FlagState};

#[derive(Debug)]
pub struct FlagRules {
    pub map_id: u32,
    pub starts: [u32; 2],
    pub capture_stat: u32,
    pub return_stat: u32,
}

const WARSONG: FlagRules = FlagRules {
    map_id: 2106,
    starts: [7051, 7050],
    capture_stat: 928,
    return_stat: 929,
};
const TWIN_PEAKS: FlagRules = FlagRules {
    map_id: 726,
    starts: [1726, 1727],
    capture_stat: 290,
    return_stat: 291,
};

pub fn flag_rules(map_id: u32) -> Option<&'static FlagRules> {
    match map_id {
        2106 => Some(&WARSONG),
        726 => Some(&TWIN_PEAKS),
        _ => None,
    }
}

/// Both scripts end at17 minutes elapsed, which includes Battleground.h
/// BG_START_DELAY_2M; Twin Peaks sets Timer4248 to start+15min.
pub const FLAG_MATCH_MS: u64 = 15 * 60 * 1000;
const FLAG_CAPTURES_ALLIANCE: u32 = 1581;
const FLAG_CAPTURES_HORDE: u32 = 1582;
const FLAG_STATE_ALLIANCE: u32 = 1545;
const FLAG_STATE_HORDE: u32 = 1546;
const FLAG_CONTROL_HORDE: u32 = 2338;
const FLAG_CONTROL_ALLIANCE: u32 = 2339;
const FLAG_CAPTURES_MAX: u32 = 17303;
const TIMER_ACTIVE: u32 = 4247;

#[derive(Debug)]
pub struct FlagBattle {
    pub rules: &'static FlagRules,
    pub flags: FlagObjective,
    last_capture: Option<BgTeam>,
    elapsed_ms: u64,
}

impl FlagBattle {
    pub fn new(rules: &'static FlagRules) -> Self {
        Self {
            rules,
            flags: FlagObjective::default(),
            last_capture: None,
            elapsed_ms: 0,
        }
    }

    pub fn capture(
        &mut self,
        team: BgTeam,
        carrier: u64,
        instance: &mut BgInstance,
        template: &BgTemplate,
    ) -> bool {
        let captured = self.flags.capture(team, carrier, instance, template);
        if captured {
            self.last_capture = Some(team);
        }
        captured
    }

    /// TC OnUpdate time limit: more captures win, equal non-zero scores go to the
    /// team that captured last, and0-0 is a tie.
    pub fn advance(&mut self, delta_ms: u64, instance: &mut BgInstance) {
        if instance.phase != BgPhase::InProgress {
            return;
        }
        self.elapsed_ms += delta_ms;
        if self.elapsed_ms < FLAG_MATCH_MS {
            return;
        }
        let (a, b) = (instance.score_a.score, instance.score_b.score);
        let outcome = match (a, b) {
            (0, 0) => BgOutcome::Draw,
            _ if a > b => BgOutcome::Winner(BgTeam::A),
            _ if b > a => BgOutcome::Winner(BgTeam::B),
            _ => self.last_capture.map_or(BgOutcome::Draw, BgOutcome::Winner),
        };
        instance.end(outcome);
    }

    pub fn world_states(&self, instance: &BgInstance) -> Vec<(u32, i32)> {
        vec![
            (FLAG_CAPTURES_ALLIANCE, instance.score_a.score as i32),
            (FLAG_CAPTURES_HORDE, instance.score_b.score as i32),
            (FLAG_CAPTURES_MAX, 3),
            (FLAG_STATE_ALLIANCE, flag_state(self.flags.flag_a)),
            (FLAG_STATE_HORDE, flag_state(self.flags.flag_b)),
            (FLAG_CONTROL_HORDE, flag_control(self.flags.flag_a)),
            (FLAG_CONTROL_ALLIANCE, flag_control(self.flags.flag_b)),
            (
                TIMER_ACTIVE,
                i32::from(instance.phase == BgPhase::InProgress),
            ),
        ]
    }
}

// GameObject.h FlagState: InBase1, Taken2, Dropped3.
fn flag_state(flag: FlagState) -> i32 {
    match flag {
        FlagState::AtBase => 1,
        FlagState::Carried { .. } => 2,
        FlagState::Dropped => 3,
    }
}

// TC UpdateFlagState: the other team's control widget is2 while taken, else1.
fn flag_control(flag: FlagState) -> i32 {
    match flag {
        FlagState::Carried { .. } => 2,
        FlagState::AtBase | FlagState::Dropped => 1,
    }
}

#[cfg(test)]
#[path = "battleground_tests/flag_maps.rs"]
mod tests;
