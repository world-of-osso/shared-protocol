//! Eye of the Storm (map 566): four presence control zones plus the neutral
//! Netherstorm flag. Rules: TC a352b1fa battleground_eye_of_the_storm.cpp and
//! GameObject.cpp ControlZone/NewFlag; positions and template fields: TDB1210.
use super::{BgInstance, BgPhase, BgTeam, BgTemplate};

#[derive(Debug)]
pub struct EyeTower {
    pub entry: u32,
    pub name: &'static str,
    pub position: [f32; 3],
    /// TC m_PointsIconStruct uncontrolled / Alliance / Horde control WorldStates.
    pub uncontrolled_state: u32,
    pub alliance_state: u32,
    pub horde_state: u32,
}

#[derive(Debug)]
pub struct EyeRules {
    pub map_id: u32,
    pub starts: [u32; 2],
    pub flag_entry: u32,
    pub capture_stat: u32,
    pub towers: [EyeTower; 4],
}

const fn tower(entry: u32, name: &'static str, position: [f32; 3], states: [u32; 3]) -> EyeTower {
    EyeTower {
        entry,
        name,
        position,
        uncontrolled_state: states[0],
        alliance_state: states[1],
        horde_state: states[2],
    }
}

const EYE: EyeRules = EyeRules {
    map_id: 566,
    starts: [1103, 1104],
    flag_entry: 208977,
    capture_stat: 183,
    towers: [
        tower(
            184081,
            "Fel Reaver Ruins",
            [2024.6, 1742.82, 1195.16],
            [2725, 2726, 2727],
        ),
        tower(
            184080,
            "Blood Elf Tower",
            [2050.49, 1372.24, 1194.56],
            [2722, 2723, 2724],
        ),
        tower(
            184083,
            "Draenei Ruins",
            [2301.01, 1386.93, 1197.18],
            [2731, 2732, 2733],
        ),
        tower(
            184082,
            "Mage Tower",
            [2282.12, 1760.01, 1189.71],
            [2728, 2730, 2729],
        ),
    ],
};

pub fn eye_rules(map_id: u32) -> Option<&'static EyeRules> {
    (map_id == EYE.map_id).then_some(&EYE)
}

// Control zone template shared by 184080-184083: Data0 radius 80, Data12 neutralPercent 40,
// Data14/15 min/maxSuperiority 1/5, Data16/17 min/maxTime 90/180 s, Data20 startingValue 50,
// Data25 UncontestedTime 0. Instanced zones beat every second.
const ZONE_RADIUS: f32 = 80.0;
const MIN_SUPERIORITY: u32 = 1;
const MAX_SUPERIORITY: u32 = 5;
const MIN_TIME_SECS: i64 = 90;
const MAX_TIME_SECS: i64 = 180;
const HEARTBEAT_MS: u64 = 1000;
/// Zone value in 1/189 points: every superiority's 100-point rate is then a whole number
/// of units per second (105, 120, 140, 168, 210), so threshold crossings are exact.
const UNITS: i64 = 189;
const FULL: i64 = 100 * UNITS;
const START: i64 = 50 * UNITS;
const HORDE_BELOW: i64 = (50 - 40 / 2) * UNITS;
const ALLIANCE_ABOVE: i64 = (50 + 40 / 2) * UNITS;

// Script: POINTS_TICK_TIME 2 s, BG_EY_TickPoints and BG_EY_FlagPoints by bases held.
const TICK_MS: u64 = 2000;
const TICK_POINTS: [u32; 4] = [1, 2, 5, 10];
const FLAG_POINTS: [u32; 4] = [75, 85, 100, 500];
/// CanCaptureFlag: nearest control zone within 40 yd of the carrier must be held.
const CAPTURE_ZONE_RANGE: f32 = 40.0;
/// Capture-flag areatrigger spawns 38-41 (create-properties 33: sphere radius 1.5).
const CAPTURE_TRIGGERS: [[f32; 3]; 4] = [
    [2043.95, 1729.78, 1190.04],
    [2048.47, 1393.59, 1194.54],
    [2284.77, 1731.11, 1190.08],
    [2286.56, 1402.34, 1197.29],
];
const TRIGGER_RADIUS: f32 = 1.5;
// Flag 208977 newflag Data7 ExpireDuration and Data8 RespawnTime.
const DROP_EXPIRE_MS: u64 = 10_000;
const RESPAWN_MS: u64 = 20_000;
// GameObject.h FlagState values carried by WorldState 8863.
const FLAG_IN_BASE: i32 = 1;
const FLAG_TAKEN: i32 = 2;
const FLAG_DROPPED: i32 = 3;
const FLAG_RESPAWNING: i32 = 4;
// BG_EY_FlagState values carried by WorldStates 9808/9809.
const UI_ON_BASE: i32 = 0;
const UI_WAIT_RESPAWN: i32 = 1;
const UI_ON_PLAYER: i32 = 2;

#[derive(Debug, Clone, Copy, PartialEq)]
enum EyeFlag {
    InBase,
    Taken { carrier: u64 },
    Dropped { position: [f32; 3], left_ms: u64 },
    Respawning { left_ms: u64 },
}

/// Where the flag object is visible; TC NewFlag hides it while taken or respawning.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FlagSite {
    Home,
    Dropped([f32; 3]),
}

#[derive(Debug)]
pub struct EyeBattle {
    pub rules: &'static EyeRules,
    zones: [i64; 4],
    flag: EyeFlag,
    /// WorldState 8863 keeps its TDB default 0 until the flag first changes state.
    flag_state: i32,
    flag_ui: [i32; 2],
    heartbeat_left_ms: u64,
    tick_left_ms: u64,
}

impl EyeBattle {
    pub fn new(rules: &'static EyeRules) -> Self {
        Self {
            rules,
            zones: [START; 4],
            flag: EyeFlag::InBase,
            flag_state: 0,
            flag_ui: [UI_ON_BASE; 2],
            heartbeat_left_ms: HEARTBEAT_MS,
            tick_left_ms: TICK_MS,
        }
    }

    /// Run heartbeats, resource ticks and flag timers in order; `players` are the alive
    /// participants' teams and WoW positions on the match map.
    pub fn advance(
        &mut self,
        mut delta_ms: u64,
        players: &[(BgTeam, [f32; 3])],
        instance: &mut BgInstance,
        template: &BgTemplate,
    ) {
        while delta_ms > 0 && instance.phase == BgPhase::InProgress {
            let step = delta_ms
                .min(self.heartbeat_left_ms)
                .min(self.tick_left_ms)
                .min(self.flag_timer().unwrap_or(u64::MAX));
            delta_ms -= step;
            self.heartbeat_left_ms -= step;
            self.tick_left_ms -= step;
            self.elapse_flag(step);
            if self.heartbeat_left_ms == 0 {
                self.heartbeat(players);
                self.heartbeat_left_ms = HEARTBEAT_MS;
            }
            if self.tick_left_ms == 0 {
                for team in [BgTeam::A, BgTeam::B] {
                    if let Some(bases) = self.base_count(team).checked_sub(1) {
                        instance.add_score(team, TICK_POINTS[bases], template);
                    }
                }
                self.tick_left_ms = TICK_MS;
            }
        }
    }

    fn heartbeat(&mut self, players: &[(BgTeam, [f32; 3])]) {
        for (tower, value) in self.rules.towers.iter().zip(&mut self.zones) {
            let inside = |team| {
                players
                    .iter()
                    .filter(|(t, at)| *t == team && distance(*at, tower.position) <= ZONE_RADIUS)
                    .count() as u32
            };
            let (alliance, horde) = (inside(BgTeam::A), inside(BgTeam::B));
            let superiority = alliance.abs_diff(horde);
            if superiority < MIN_SUPERIORITY {
                continue;
            }
            let gained = rate_per_heartbeat(superiority);
            let signed = if alliance > horde { gained } else { -gained };
            *value = (*value + signed).clamp(0, FULL);
        }
    }

    fn flag_timer(&self) -> Option<u64> {
        match self.flag {
            EyeFlag::Dropped { left_ms, .. } | EyeFlag::Respawning { left_ms } => Some(left_ms),
            _ => None,
        }
    }

    fn elapse_flag(&mut self, step: u64) {
        let (EyeFlag::Dropped { left_ms, .. } | EyeFlag::Respawning { left_ms }) = &mut self.flag
        else {
            return;
        };
        *left_ms -= step;
        if *left_ms == 0 {
            self.flag = EyeFlag::InBase;
            self.flag_state = FLAG_IN_BASE;
        }
    }

    fn owner(&self, index: usize) -> Option<BgTeam> {
        match self.zones[index] {
            value if value < HORDE_BELOW => Some(BgTeam::B),
            value if value > ALLIANCE_ABOVE => Some(BgTeam::A),
            _ => None,
        }
    }

    fn base_count(&self, team: BgTeam) -> usize {
        (0..self.zones.len())
            .filter(|&index| self.owner(index) == Some(team))
            .count()
    }

    /// Take the flag from its stand or from the ground; either team may take a dropped flag.
    pub fn pickup(&mut self, team: BgTeam, carrier: u64) -> bool {
        if !matches!(self.flag, EyeFlag::InBase | EyeFlag::Dropped { .. }) {
            return false;
        }
        self.flag = EyeFlag::Taken { carrier };
        self.flag_state = FLAG_TAKEN;
        self.flag_ui[team_index(team)] = UI_ON_PLAYER;
        true
    }

    pub fn carrier(&self) -> Option<u64> {
        match self.flag {
            EyeFlag::Taken { carrier } => Some(carrier),
            _ => None,
        }
    }

    /// The carrier died or left: the flag lies at `position` until it expires home.
    pub fn drop_flag(&mut self, position: [f32; 3]) {
        if self.carrier().is_none() {
            return;
        }
        self.flag = EyeFlag::Dropped {
            position,
            left_ms: DROP_EXPIRE_MS,
        };
        self.flag_state = FLAG_DROPPED;
        self.flag_ui = [UI_WAIT_RESPAWN; 2];
    }

    /// Score when the carrier stands in a capture trigger whose nearest zone its team holds.
    pub fn capture(
        &mut self,
        carrier: u64,
        team: BgTeam,
        position: [f32; 3],
        instance: &mut BgInstance,
        template: &BgTemplate,
    ) -> bool {
        if self.carrier() != Some(carrier)
            || !CAPTURE_TRIGGERS
                .iter()
                .any(|trigger| distance(*trigger, position) <= TRIGGER_RADIUS)
        {
            return false;
        }
        let nearest = (0..self.zones.len())
            .map(|index| (index, distance(self.rules.towers[index].position, position)))
            .filter(|(_, range)| *range <= CAPTURE_ZONE_RANGE)
            .min_by(|a, b| a.1.total_cmp(&b.1));
        if nearest.is_none_or(|(index, _)| self.owner(index) != Some(team)) {
            return false;
        }
        let bases = self.base_count(team);
        instance.add_score(team, FLAG_POINTS[bases - 1], template);
        self.flag = EyeFlag::Respawning {
            left_ms: RESPAWN_MS,
        };
        self.flag_state = FLAG_RESPAWNING;
        self.flag_ui = [UI_ON_BASE; 2];
        true
    }

    pub fn flag_site(&self) -> Option<FlagSite> {
        match self.flag {
            EyeFlag::InBase => Some(FlagSite::Home),
            EyeFlag::Dropped { position, .. } => Some(FlagSite::Dropped(position)),
            EyeFlag::Taken { .. } | EyeFlag::Respawning { .. } => None,
        }
    }

    pub fn world_states(&self, instance: &BgInstance) -> Vec<(u32, i32)> {
        let mut values = vec![
            (1776, instance.score_a.score as i32),
            (1777, instance.score_b.score as i32),
            (1780, 1500),
            (2752, self.base_count(BgTeam::A) as i32),
            (2753, self.base_count(BgTeam::B) as i32),
            (8863, self.flag_state),
            (9808, self.flag_ui[0]),
            (9809, self.flag_ui[1]),
        ];
        for (index, tower) in self.rules.towers.iter().enumerate() {
            let owner = self.owner(index);
            values.push((tower.uncontrolled_state, i32::from(owner.is_none())));
            values.push((tower.alliance_state, i32::from(owner == Some(BgTeam::A))));
            values.push((tower.horde_state, i32::from(owner == Some(BgTeam::B))));
        }
        values
    }
}

/// Seconds per 100 points fall linearly from maxTime at minSuperiority to minTime at
/// maxSuperiority and stay there. TC computes `minTime - maxTime` in uint32 (wrapping) and
/// does not saturate; this follows the template fields' evident intent instead.
fn rate_per_heartbeat(superiority: u32) -> i64 {
    let span = i64::from(MAX_SUPERIORITY - MIN_SUPERIORITY);
    let over = i64::from(superiority.clamp(MIN_SUPERIORITY, MAX_SUPERIORITY) - MIN_SUPERIORITY);
    let scaled_seconds = MAX_TIME_SECS * span - (MAX_TIME_SECS - MIN_TIME_SECS) * over;
    debug_assert_eq!(FULL * span % scaled_seconds, 0);
    FULL * span / scaled_seconds
}

fn distance(a: [f32; 3], b: [f32; 3]) -> f32 {
    a.iter()
        .zip(b)
        .map(|(x, y)| (x - y) * (x - y))
        .sum::<f32>()
        .sqrt()
}

fn team_index(team: BgTeam) -> usize {
    match team {
        BgTeam::A => 0,
        BgTeam::B => 1,
    }
}

#[cfg(test)]
#[path = "battleground_tests/eye_of_the_storm.rs"]
mod tests;
