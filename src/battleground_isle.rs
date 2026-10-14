//! Isle of Conquest source rules: TC a352b1fa battleground_isle_of_conquest.cpp,
//! isle_of_conquest.cpp (`go_ioc_contested_object` captures one minute after it spawns)
//! and TDB1210 spawn groups 1201-1237 / world_state defaults for map 628.
use super::*;

const CAPTURE_MS: u64 = 60_000;
/// `IOC_RESOURCE_TIMER`: owned, uncontested Refinery and Quarry add one reinforcement.
const RESOURCE_TICK_MS: u64 = 45_000;
const MAX_REINFORCEMENTS: i32 = 400;
const BASES_ASSAULTED: u32 = 245;
const BASES_DEFENDED: u32 = 246;
/// `NPC_HIGH_COMMANDER_HALFORD_WYRMBANE` (Alliance boss) and `NPC_OVERLORD_AGMAR`.
const ALLIANCE_BOSS: u32 = 34924;
const HORDE_BOSS: u32 = 34922;

#[derive(Debug, Clone, Copy)]
struct NodeSpec {
    /// Alliance controlled, Alliance contested, Horde controlled, Horde contested, neutral.
    banners: [u32; 5],
    /// Uncontrolled, ConflictA, ConflictH, ControlledA, ControlledH.
    world_states: [u32; 5],
    initial: Option<BgTeam>,
    /// `i < NODE_TYPE_DOCKS`: Refinery and Quarry tick reinforcements.
    resource: bool,
}

const fn node(
    banners: [u32; 5],
    world_states: [u32; 5],
    initial: Option<BgTeam>,
    resource: bool,
) -> NodeSpec {
    NodeSpec {
        banners,
        world_states,
        initial,
        resource,
    }
}

/// TC `nodePointInitial` order; banners are the TDB spawn group objects per state.
const NODES: [NodeSpec; 7] = [
    // Refinery
    node(
        [195339, 195340, 195341, 195342, 195343],
        [4311, 4315, 4312, 4314, 4313],
        None,
        true,
    ),
    // Quarry
    node(
        [195334, 195335, 195336, 195337, 195338],
        [4306, 4310, 4307, 4309, 4308],
        None,
        true,
    ),
    // Docks
    node(
        [195149, 195150, 195151, 195152, 195157],
        [4301, 4305, 4302, 4304, 4303],
        None,
        false,
    ),
    // Hangar
    node(
        [195153, 195154, 195155, 195156, 195158],
        [4296, 4300, 4297, 4299, 4298],
        None,
        false,
    ),
    // Workshop
    node(
        [195132, 195144, 195130, 195145, 195133],
        [4294, 4228, 4293, 4229, 4230],
        None,
        false,
    ),
    // Alliance Keep graveyard
    node(
        [195396, 195397, 195398, 195399, 0],
        [4341, 4342, 4343, 4339, 4340],
        Some(BgTeam::A),
        false,
    ),
    // Horde Keep graveyard
    node(
        [195391, 195392, 195393, 195394, 0],
        [4346, 4347, 4348, 4344, 4345],
        Some(BgTeam::B),
        false,
    ),
];

#[derive(Debug, Clone, Copy)]
struct NodeState {
    /// `ControlledX` or `ConflictX` team; `None` is `Neutral`.
    team: Option<BgTeam>,
    contested: bool,
    last_controlled: Option<BgTeam>,
    capture_left_ms: u64,
}

impl NodeState {
    fn initial(spec: NodeSpec) -> Self {
        Self {
            team: spec.initial,
            contested: false,
            last_controlled: spec.initial,
            capture_left_ms: 0,
        }
    }

    /// `OnPlayerInteractWithBanner`.
    fn interact(&mut self, team: BgTeam) -> Result<u32, &'static str> {
        let defend = self.contested && self.last_controlled == Some(team);
        let assault = match (self.team, self.contested) {
            (None, _) => true,
            (Some(owner), false) => owner != team,
            (Some(assaulter), true) => !defend && assaulter != team,
        };
        if defend {
            *self = Self {
                team: Some(team),
                contested: false,
                last_controlled: Some(team),
                capture_left_ms: 0,
            };
            return Ok(BASES_DEFENDED);
        }
        if !assault {
            return Err("objective cannot be used by this team now");
        }
        self.team = Some(team);
        self.contested = true;
        self.capture_left_ms = CAPTURE_MS;
        Ok(BASES_ASSAULTED)
    }

    /// `ACTION_IOC_CAPTURE_CAPTURABLE_OBJECT` one minute after the contested banner.
    fn advance(&mut self, delta_ms: u64) {
        if !self.contested {
            return;
        }
        self.capture_left_ms = self.capture_left_ms.saturating_sub(delta_ms);
        if self.capture_left_ms == 0 {
            self.contested = false;
            self.last_controlled = self.team;
        }
    }

    fn banner(&self, spec: NodeSpec) -> Option<u32> {
        let index = match (self.team, self.contested) {
            (Some(BgTeam::A), false) => 0,
            (Some(BgTeam::A), true) => 1,
            (Some(BgTeam::B), false) => 2,
            (Some(BgTeam::B), true) => 3,
            (None, _) => 4,
        };
        let entry = spec.banners[index];
        (entry != 0).then_some(entry)
    }

    /// `UpdateNodeWorldState`.
    fn world_states(&self, spec: NodeSpec) -> [(u32, i32); 5] {
        let is = |team, contested| self.team == Some(team) && self.contested == contested;
        let values = [
            self.team.is_none(),
            is(BgTeam::A, true),
            is(BgTeam::B, true),
            is(BgTeam::A, false),
            is(BgTeam::B, false),
        ];
        std::array::from_fn(|index| (spec.world_states[index], i32::from(values[index])))
    }
}

#[derive(Debug, Clone, Copy)]
struct GateSpec {
    entry: u32,
    team: BgTeam,
    closed_state: u32,
    open_state: u32,
}

const fn gate(entry: u32, team: BgTeam, closed_state: u32, open_state: u32) -> GateSpec {
    GateSpec {
        entry,
        team,
        closed_state,
        open_state,
    }
}

/// TC GetWorldStateFromGateEntry and GetGateIDFromEntry, Front/West/East per keep.
const GATES: [GateSpec; 6] = [
    gate(195494, BgTeam::B, 4317, 4322),
    gate(195495, BgTeam::B, 4318, 4321),
    gate(195496, BgTeam::B, 4319, 4320),
    gate(195698, BgTeam::A, 4328, 4323),
    gate(195699, BgTeam::A, 4327, 4324),
    gate(195700, BgTeam::A, 4326, 4325),
];

#[derive(Debug)]
pub struct IsleBattle {
    nodes: [NodeState; 7],
    destroyed_gates: [bool; 6],
    resource_elapsed_ms: u64,
}

impl Default for IsleBattle {
    fn default() -> Self {
        Self {
            nodes: NODES.map(NodeState::initial),
            destroyed_gates: [false; 6],
            resource_elapsed_ms: 0,
        }
    }
}

impl IsleBattle {
    /// Apply a completed normal Opening use; return its source scoreboard stat.
    pub fn interact(
        &mut self,
        entry: u32,
        team: BgTeam,
        instance: &BgInstance,
    ) -> Result<u32, &'static str> {
        let node = self.current_banner_node(entry, instance)?;
        self.nodes[node].interact(team)
    }

    pub fn check_interact(
        &self,
        entry: u32,
        team: BgTeam,
        instance: &BgInstance,
    ) -> Result<(), &'static str> {
        let node = self.current_banner_node(entry, instance)?;
        let mut probe = self.nodes[node];
        probe.interact(team).map(|_| ())
    }

    fn current_banner_node(
        &self,
        entry: u32,
        instance: &BgInstance,
    ) -> Result<usize, &'static str> {
        if instance.bg_id != 30 || instance.phase != BgPhase::InProgress {
            return Err("no active Isle of Conquest match");
        }
        let node = (0..NODES.len())
            .find(|&node| self.nodes[node].banner(NODES[node]) == Some(entry))
            .ok_or("banner is not the current Isle of Conquest objective")?;
        let keep = match node {
            5 => Some(BgTeam::A),
            6 => Some(BgTeam::B),
            _ => None,
        };
        if keep.is_some_and(|team| !self.keep_is_breached(team)) {
            return Err("keep gates have not been breached");
        }
        Ok(node)
    }

    /// A source destructible-building death opens its keep, not an admin outcome.
    pub fn gate_destroyed(
        &mut self,
        entry: u32,
        instance: &BgInstance,
    ) -> Result<BgTeam, &'static str> {
        if instance.bg_id != 30 || instance.phase != BgPhase::InProgress {
            return Err("no active Isle of Conquest match");
        }
        let index = GATES
            .iter()
            .position(|gate| gate.entry == entry)
            .ok_or("object is not an Isle of Conquest gate")?;
        self.destroyed_gates[index] = true;
        Ok(GATES[index].team)
    }

    pub fn keep_is_breached(&self, team: BgTeam) -> bool {
        GATES
            .iter()
            .zip(self.destroyed_gates)
            .any(|(gate, destroyed)| gate.team == team && destroyed)
    }

    pub fn advance(&mut self, delta_ms: u64, instance: &mut BgInstance) {
        if instance.bg_id != 30 || instance.phase != BgPhase::InProgress {
            return;
        }
        for node in &mut self.nodes {
            node.advance(delta_ms);
        }
        self.resource_elapsed_ms += delta_ms;
        while self.resource_elapsed_ms >= RESOURCE_TICK_MS {
            self.resource_elapsed_ms -= RESOURCE_TICK_MS;
            for (node, _) in self
                .nodes
                .iter()
                .zip(NODES)
                .filter(|(_, spec)| spec.resource)
            {
                match (node.last_controlled, node.contested) {
                    (Some(BgTeam::A), false) => instance.score_a.reinforcements += 1,
                    (Some(BgTeam::B), false) => instance.score_b.reinforcements += 1,
                    _ => {}
                }
            }
        }
    }

    /// `OnUnitKilled`: a keep boss's death ends the match for the other team.
    pub fn npc_killed(&mut self, entry: u32, instance: &mut BgInstance) -> bool {
        if instance.bg_id != 30 || instance.phase != BgPhase::InProgress {
            return false;
        }
        let winner = match entry {
            ALLIANCE_BOSS => BgTeam::B,
            HORDE_BOSS => BgTeam::A,
            _ => return false,
        };
        instance.end(BgOutcome::Winner(winner));
        true
    }

    pub fn world_states(&self, instance: &BgInstance) -> Vec<(u32, i32)> {
        let mut states: Vec<_> = self
            .nodes
            .iter()
            .zip(NODES)
            .flat_map(|(node, spec)| node.world_states(spec))
            .collect();
        states.extend(
            GATES
                .iter()
                .zip(self.destroyed_gates)
                .flat_map(|(gate, destroyed)| {
                    [
                        (gate.closed_state, i32::from(!destroyed)),
                        (gate.open_state, i32::from(destroyed)),
                    ]
                }),
        );
        states.extend([
            (4221, 1),
            (4222, 1),
            (4226, instance.score_a.reinforcements as i32),
            (4227, instance.score_b.reinforcements as i32),
            (17377, MAX_REINFORCEMENTS),
        ]);
        states
    }
}

pub fn is_isle_banner_entry(entry: u32) -> bool {
    entry != 0 && NODES.iter().any(|spec| spec.banners.contains(&entry))
}

pub fn isle_gate_entries() -> Vec<u32> {
    GATES.iter().map(|gate| gate.entry).collect()
}

pub fn isle_keep_door_entries(team: BgTeam) -> &'static [u32] {
    match team {
        BgTeam::A => &[195451, 195452],
        BgTeam::B => &[195223],
    }
}

pub fn isle_banner_entries() -> Vec<u32> {
    NODES
        .into_iter()
        .flat_map(|spec| spec.banners)
        .filter(|entry| *entry != 0)
        .collect()
}
