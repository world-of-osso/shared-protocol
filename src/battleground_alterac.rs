//! Alterac source rules: TC a352b1fa battleground_alterac_valley.cpp and alterac_valley.cpp.
use super::{BgInstance, BgOutcome, BgPhase, BgTeam};

pub const ALTERAC_OPENING_SPELL: u32 = 21651;
pub const ALTERAC_NODE_COUNT: usize = 15;
const CAPTURE_MS: u64 = 240_000;
const MINE_TICK_MS: u64 = 45_000;
const TOWER_LOSS: u32 = 75;
const CAPTAIN_LOSS: u32 = 100;

#[derive(Debug, Clone, Copy)]
struct NodeSpec {
    // ControlledA, contestedA, controlledH, contestedH, neutral; zero means no variant.
    banners: [u32; 5],
    // Source BGAVNodeInfo: controlledA, assaultedA, controlledH, assaultedH, owner.
    world_states: [u32; 5],
    initial_owner: Option<BgTeam>,
    tower: bool,
}

const fn grave(banners: [u32; 5], world_states: [u32; 5], owner: Option<BgTeam>) -> NodeSpec {
    NodeSpec {
        banners,
        world_states,
        initial_owner: owner,
        tower: false,
    }
}
const fn tower(banners: [u32; 5], world_states: [u32; 5], owner: BgTeam) -> NodeSpec {
    NodeSpec {
        banners,
        world_states,
        initial_owner: Some(owner),
        tower: true,
    }
}

const NODES: [NodeSpec; ALTERAC_NODE_COUNT] = [
    grave(
        [179465, 179466, 179467, 179468, 0],
        [1325, 1326, 1327, 1328, 0],
        Some(BgTeam::A),
    ),
    grave(
        [178389, 179286, 178388, 179287, 0],
        [1333, 1335, 1334, 1336, 0],
        Some(BgTeam::A),
    ),
    grave(
        [179284, 179308, 179285, 179310, 0],
        [1302, 1304, 1301, 1303, 0],
        Some(BgTeam::A),
    ),
    grave(
        [178365, 180419, 178364, 180420, 180418],
        [1341, 1343, 1342, 1344, 0],
        None,
    ),
    grave(
        [179481, 179482, 179483, 179484, 0],
        [1346, 1348, 1347, 1349, 0],
        Some(BgTeam::B),
    ),
    grave(
        [178394, 179304, 178393, 179305, 0],
        [1337, 1339, 1338, 1340, 0],
        Some(BgTeam::B),
    ),
    grave(
        [179470, 179471, 179472, 179473, 0],
        [1329, 1331, 1330, 1332, 0],
        Some(BgTeam::B),
    ),
    tower(
        [178925, 0, 0, 179435, 0],
        [1361, 1375, 1370, 1378, 1181],
        BgTeam::A,
    ),
    tower(
        [178929, 0, 0, 179439, 0],
        [1362, 1374, 1371, 1379, 1182],
        BgTeam::A,
    ),
    tower(
        [178935, 0, 0, 179441, 0],
        [1363, 1376, 1372, 1380, 1183],
        BgTeam::A,
    ),
    tower(
        [178936, 0, 0, 179443, 0],
        [1364, 1377, 1373, 1381, 1184],
        BgTeam::A,
    ),
    tower(
        [0, 178940, 178946, 0, 0],
        [1368, 1390, 1385, 1395, 1188],
        BgTeam::B,
    ),
    tower(
        [0, 179453, 178945, 0, 0],
        [1367, 1389, 1384, 1394, 1187],
        BgTeam::B,
    ),
    tower(
        [0, 179449, 178944, 0, 0],
        [1366, 1388, 1383, 1393, 1186],
        BgTeam::B,
    ),
    tower(
        [0, 179445, 178943, 0, 0],
        [1365, 1387, 1382, 1392, 1185],
        BgTeam::B,
    ),
];

#[derive(Debug, Clone, Copy)]
struct NodeState {
    owner: Option<BgTeam>,
    total_owner: Option<BgTeam>,
    contested: bool,
    destroyed: bool,
    capture_left_ms: u64,
}

impl NodeState {
    fn initial(spec: NodeSpec) -> Self {
        Self {
            owner: spec.initial_owner,
            total_owner: spec.initial_owner,
            contested: false,
            destroyed: false,
            capture_left_ms: 0,
        }
    }

    fn interact(&mut self, spec: NodeSpec, team: BgTeam) -> Result<u32, &'static str> {
        if self.destroyed || self.owner == Some(team) {
            return Err("objective cannot be assaulted by its current owner");
        }
        let defense = self.contested && self.total_owner == Some(team);
        self.owner = Some(team);
        self.contested = !defense;
        self.capture_left_ms = if defense { 0 } else { CAPTURE_MS };
        Ok(match (spec.tower, defense) {
            (true, false) => 61,
            (false, false) => 63,
            (true, true) => 64,
            (false, true) => 65,
        })
    }

    fn advance(&mut self, delta_ms: u64, tower: bool) -> bool {
        if !self.contested {
            return false;
        }
        self.capture_left_ms = self.capture_left_ms.saturating_sub(delta_ms);
        if self.capture_left_ms > 0 {
            return false;
        }
        self.contested = false;
        self.total_owner = self.owner;
        self.destroyed = tower;
        true
    }

    fn banner(&self, spec: NodeSpec) -> Option<u32> {
        if self.destroyed {
            return None;
        }
        let index = match (self.owner, self.contested) {
            (Some(BgTeam::A), false) => 0,
            (Some(BgTeam::A), true) => 1,
            (Some(BgTeam::B), false) => 2,
            (Some(BgTeam::B), true) => 3,
            (None, _) => 4,
        };
        let entry = spec.banners[index];
        (entry != 0).then_some(entry)
    }

    fn world_states(&self, spec: NodeSpec) -> Vec<(u32, i32)> {
        let alliance = self.owner == Some(BgTeam::A);
        let horde = self.owner == Some(BgTeam::B);
        let values = [
            i32::from(alliance && !self.contested),
            i32::from(alliance && self.contested),
            i32::from(horde && !self.contested),
            i32::from(horde && self.contested),
            owner_value(self.owner),
        ];
        spec.world_states
            .into_iter()
            .zip(values)
            .filter(|(id, _)| *id != 0)
            .collect()
    }
}

#[derive(Debug)]
pub struct AlteracBattle {
    nodes: [NodeState; ALTERAC_NODE_COUNT],
    mines: [Option<BgTeam>; 2],
    mine_elapsed_ms: u64,
    captains_alive: [bool; 2],
    generals_alive: [bool; 2],
}

impl Default for AlteracBattle {
    fn default() -> Self {
        Self {
            nodes: NODES.map(NodeState::initial),
            mines: [None; 2],
            mine_elapsed_ms: 0,
            captains_alive: [true; 2],
            generals_alive: [true; 2],
        }
    }
}

impl AlteracBattle {
    /// Apply a completed normal Opening use; return its source scoreboard stat.
    pub fn interact(
        &mut self,
        entry: u32,
        team: BgTeam,
        instance: &BgInstance,
    ) -> Result<u32, &'static str> {
        let node = self.find_usable_banner(entry, team, instance)?;
        self.nodes[node].interact(NODES[node], team)
    }

    pub fn check_interact(
        &self,
        entry: u32,
        team: BgTeam,
        instance: &BgInstance,
    ) -> Result<(), &'static str> {
        self.find_usable_banner(entry, team, instance).map(|_| ())
    }

    fn find_usable_banner(
        &self,
        entry: u32,
        team: BgTeam,
        instance: &BgInstance,
    ) -> Result<usize, &'static str> {
        if instance.bg_id != 1 || instance.phase != BgPhase::InProgress {
            return Err("no active Alterac match");
        }
        let node = (0..NODES.len())
            .find(|index| self.banner_entry(*index) == Some(entry))
            .ok_or("banner is not the current Alterac objective")?;
        if self.nodes[node].owner == Some(team) {
            return Err("objective belongs to this team");
        }
        Ok(node)
    }

    pub fn banner_entry(&self, node: usize) -> Option<u32> {
        self.nodes.get(node)?.banner(*NODES.get(node)?)
    }

    /// Completed tower destructions attributed to each assaulting team (A/B).
    pub fn destroyed_towers(&self) -> [u32; 2] {
        [BgTeam::A, BgTeam::B].map(|team| {
            self.nodes
                .iter()
                .filter(|node| node.destroyed && node.owner == Some(team))
                .count() as u32
        })
    }

    pub fn advance(&mut self, delta_ms: u64, instance: &mut BgInstance) {
        if instance.bg_id != 1 || instance.phase != BgPhase::InProgress {
            return;
        }
        for (node, spec) in self.nodes.iter_mut().zip(NODES) {
            if instance.phase != BgPhase::InProgress {
                break;
            }
            if node.advance(delta_ms, spec.tower) && spec.tower {
                instance.deduct_reinforcements(
                    opponent(node.owner.expect("assault has an owner")),
                    TOWER_LOSS,
                );
            }
        }
        self.advance_mines(delta_ms, instance);
    }

    fn advance_mines(&mut self, delta_ms: u64, instance: &mut BgInstance) {
        if instance.phase != BgPhase::InProgress {
            return;
        }
        self.mine_elapsed_ms += delta_ms;
        while self.mine_elapsed_ms >= MINE_TICK_MS {
            self.mine_elapsed_ms -= MINE_TICK_MS;
            for team in self.mines.into_iter().flatten() {
                match team {
                    BgTeam::A => instance.score_a.reinforcements += 1,
                    BgTeam::B => instance.score_b.reinforcements += 1,
                }
            }
        }
    }

    pub fn npc_killed(
        &mut self,
        entry: u32,
        killer: Option<BgTeam>,
        instance: &mut BgInstance,
    ) -> bool {
        if instance.bg_id != 1 || instance.phase != BgPhase::InProgress {
            return false;
        }
        match entry {
            11948 => self.general_killed(BgTeam::A, instance),
            11946 => self.general_killed(BgTeam::B, instance),
            11949 => self.captain_killed(BgTeam::A, instance),
            11947 => self.captain_killed(BgTeam::B, instance),
            11657 => self.mine_killed(0, true, killer),
            13078 | 13079 => self.mine_killed(0, false, killer),
            11677 => self.mine_killed(1, true, killer),
            13086 | 13088 => self.mine_killed(1, false, killer),
            _ => false,
        }
    }

    fn general_killed(&mut self, team: BgTeam, instance: &mut BgInstance) -> bool {
        self.generals_alive[team_index(team)] = false;
        instance.end(BgOutcome::Winner(opponent(team)));
        true
    }

    fn captain_killed(&mut self, team: BgTeam, instance: &mut BgInstance) -> bool {
        let alive = &mut self.captains_alive[team_index(team)];
        if !*alive {
            return false;
        }
        *alive = false;
        instance.deduct_reinforcements(team, CAPTAIN_LOSS);
        true
    }

    fn mine_killed(&mut self, mine: usize, neutral_leader: bool, killer: Option<BgTeam>) -> bool {
        let Some(team) = killer else {
            return false;
        };
        if neutral_leader && self.mines[mine].is_some() || self.mines[mine] == Some(team) {
            return false;
        }
        self.mines[mine] = Some(team);
        true
    }

    pub fn world_states(&self, instance: &BgInstance) -> Vec<(u32, i32)> {
        let mut states: Vec<_> = self
            .nodes
            .iter()
            .zip(NODES)
            .flat_map(|(node, spec)| node.world_states(spec))
            .collect();
        states.extend([
            (3127, instance.score_a.reinforcements as i32),
            (3128, instance.score_b.reinforcements as i32),
            (3133, 1),
            (3134, 1),
            (3136, 700),
            (601, i32::from(self.generals_alive[1])),
            (602, i32::from(self.generals_alive[0])),
            (1351, i32::from(self.captains_alive[0])),
            (1352, i32::from(self.captains_alive[1])),
            (1966, i32::from(self.nodes[3].owner.is_none())),
        ]);
        states.extend(mine_world_states(self.mines[0], [801, 1358, 1359, 1360]));
        states.extend(mine_world_states(self.mines[1], [804, 1355, 1356, 1357]));
        states
    }
}

pub fn is_alterac_banner_entry(entry: u32) -> bool {
    entry != 0 && NODES.iter().any(|spec| spec.banners.contains(&entry))
}

pub fn alterac_banner_entries() -> Vec<u32> {
    NODES
        .into_iter()
        .flat_map(|spec| spec.banners)
        .filter(|entry| *entry != 0)
        .collect()
}

fn mine_world_states(owner: Option<BgTeam>, ids: [u32; 4]) -> [(u32, i32); 4] {
    [
        (ids[0], owner_value(owner)),
        (ids[1], i32::from(owner == Some(BgTeam::A))),
        (ids[2], i32::from(owner == Some(BgTeam::B))),
        (ids[3], i32::from(owner.is_none())),
    ]
}
fn owner_value(owner: Option<BgTeam>) -> i32 {
    match owner {
        Some(BgTeam::A) => 1,
        Some(BgTeam::B) => 2,
        None => 0,
    }
}
fn team_index(team: BgTeam) -> usize {
    match team {
        BgTeam::A => 0,
        BgTeam::B => 1,
    }
}
fn opponent(team: BgTeam) -> BgTeam {
    match team {
        BgTeam::A => BgTeam::B,
        BgTeam::B => BgTeam::A,
    }
}
