use serde::{Deserialize, Serialize};

#[path = "battleground_brackets.rs"]
mod brackets;
pub use brackets::source_level_bracket;

/// Type of PvP objective for a battleground.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BgObjective {
    /// Defeat every living member of the opposing arena team.
    Elimination,
    /// Capture the enemy flag and return it to your base (e.g. Warsong Gulch).
    FlagCapture,
    /// Control territory nodes that tick score over time (e.g. Arathi Basin).
    NodeControl,
    /// Deplete the enemy team's reinforcement count (e.g. Alterac Valley).
    Reinforcements,
    /// Carry Orbs of Power for location-dependent victory points.
    PowerOrbs,
    /// Control moving mine carts and escort them to authored depots.
    Payload,
    /// Collect spawned resources before the opposing team.
    ResourceRace,
}

/// Level bracket for battleground matchmaking.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LevelBracket {
    pub min_level: u8,
    pub max_level: u8,
}

impl LevelBracket {
    pub const fn new(min_level: u8, max_level: u8) -> Self {
        Self {
            min_level,
            max_level,
        }
    }

    /// Whether a player level falls within this bracket.
    pub fn contains(&self, level: u8) -> bool {
        level >= self.min_level && level <= self.max_level
    }
}

/// Static definition of a battleground type.
///
/// Immutable data loaded at startup. Describes the rules and parameters for
/// one kind of BG (e.g. "Warsong Gulch", "Arathi Basin").
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BgTemplate {
    /// Unique battleground type ID.
    pub id: u32,
    /// Display name.
    pub name: String,
    /// Map ID for the battleground instance.
    pub map_id: u32,
    /// Players per team.
    pub team_size: u8,
    /// Minimum players per team to start (below this, match waits).
    pub min_players_per_team: u8,
    /// Primary objective type.
    pub objective: BgObjective,
    /// Maximum match duration in seconds; zero means no timed end.
    pub max_duration_secs: u32,
    /// Score required to win (flag captures, resource points, etc.).
    pub win_score: u32,
    /// Starting reinforcement count (only for `BgObjective::Reinforcements`).
    pub starting_reinforcements: u32,
    /// Level bracket for matchmaking.
    pub bracket: LevelBracket,
}

// -- Built-in BG templates --

/// Mainline Warsong Gulch — local BattlemasterList row1014 (build12.1.0.69933),
/// map2106 from TDB1210 start locations7051/7050. Three captures: world_state17303.
/// The legacy duration below remains unsourced; lifecycle callers must not use it.
pub fn warsong_gulch() -> BgTemplate {
    BgTemplate {
        id: 1014,
        name: "Warsong Gulch".into(),
        map_id: 2106,
        team_size: 10,
        min_players_per_team: 5,
        objective: BgObjective::FlagCapture,
        max_duration_secs: 25 * 60,
        win_score: 3,
        starting_reinforcements: 0,
        bracket: LevelBracket::new(10, 90),
    }
}

/// Arathi Basin — 15v15 node control (5 bases, 1500 resources to win).
/// BattlemasterList 1018/map 2107, build 12.1.0.69933; TDB1210 start 7056/7055.
/// Resource limit: Map.csv map 2107 PvpLongDescription and world_state 1780.
pub fn arathi_basin() -> BgTemplate {
    BgTemplate {
        id: 1018,
        name: "Arathi Basin".into(),
        map_id: 2107,
        team_size: 15,
        min_players_per_team: 8,
        objective: BgObjective::NodeControl,
        max_duration_secs: 0,
        win_score: 1500,
        starting_reinforcements: 0,
        bracket: LevelBracket::new(10, 90),
    }
}

/// Mainline BattlemasterList1/map30: 10–40/team, levels20–90.
/// TC a352b1fa battleground_alterac_valley.cpp starts each team at700.
pub fn alterac_valley() -> BgTemplate {
    BgTemplate {
        id: 1,
        name: "Alterac Valley".into(),
        map_id: 30,
        team_size: 40,
        min_players_per_team: 10,
        objective: BgObjective::Reinforcements,
        max_duration_secs: 0,
        win_score: 0,
        starting_reinforcements: 700,
        bracket: LevelBracket::new(20, 90),
    }
}

/// Mainline BattlemasterList30/map628: 10–40/team, levels20–90.
/// TC a352b1fa battleground_isle_of_conquest.cpp starts each team at400.
pub fn isle_of_conquest() -> BgTemplate {
    BgTemplate {
        id: 30,
        name: "Isle of Conquest".into(),
        map_id: 628,
        team_size: 40,
        min_players_per_team: 10,
        objective: BgObjective::Reinforcements,
        max_duration_secs: 0,
        win_score: 0,
        starting_reinforcements: 400,
        bracket: LevelBracket::new(20, 90),
    }
}

/// Mainline BattlemasterList 120/map 761, TDB starts 1798/1799; world_state 1780.
pub fn battle_for_gilneas() -> BgTemplate {
    BgTemplate {
        id: 120,
        name: "The Battle for Gilneas".into(),
        map_id: 761,
        team_size: 10,
        min_players_per_team: 3,
        objective: BgObjective::NodeControl,
        max_duration_secs: 0,
        win_score: 1500,
        starting_reinforcements: 0,
        bracket: LevelBracket::new(20, 90),
    }
}

/// BattlemasterList 108 (12.1.0.69933): Capture the Flag, 5–10 players, levels 30–90;
/// BattlemasterListXMap 623 → map 726; TDB starts 1726/1727. Three captures win.
pub fn twin_peaks() -> BgTemplate {
    BgTemplate {
        id: 108,
        name: "Twin Peaks".into(),
        map_id: 726,
        team_size: 10,
        min_players_per_team: 5,
        objective: BgObjective::FlagCapture,
        max_duration_secs: 0,
        win_score: 3,
        starting_reinforcements: 0,
        bracket: LevelBracket::new(30, 90),
    }
}

/// BattlemasterList 7 (12.1.0.69933): Domination, 8–15 players, levels 20–90;
/// BattlemasterListXMap 370 → map 566; TDB starts 1103/1104. Map.csv: earn 1500 resources.
pub fn eye_of_the_storm() -> BgTemplate {
    BgTemplate {
        id: 7,
        name: "Eye of the Storm".into(),
        map_id: 566,
        team_size: 15,
        min_players_per_team: 8,
        objective: BgObjective::NodeControl,
        max_duration_secs: 0,
        win_score: 1500,
        starting_reinforcements: 0,
        bracket: LevelBracket::new(20, 90),
    }
}

/// BattlemasterList 699, local 12.1.0.69933: Power Orbs, 2–10 per team, levels 40–90.
/// TDB1210 starts 4059/4060 map 998; WorldState 17388 caps victory points at 1500.
pub fn temple_of_kotmogu() -> BgTemplate {
    BgTemplate {
        id: 699,
        name: "Temple of Kotmogu".into(),
        map_id: 998,
        team_size: 10,
        min_players_per_team: 2,
        objective: BgObjective::PowerOrbs,
        max_duration_secs: 0,
        win_score: 1500,
        starting_reinforcements: 0,
        bracket: LevelBracket::new(40, 90),
    }
}

/// Local BattlemasterList 708/XMap 627: Payload map 727, 3–10 per team, levels 30–90.
/// Pinned TC SilvershardMines::Score::Max is 1500; no unsourced match-duration timer.
pub fn silvershard_mines() -> BgTemplate {
    BgTemplate {
        id: 708,
        name: "Silvershard Mines".into(),
        map_id: 727,
        team_size: 10,
        min_players_per_team: 3,
        objective: BgObjective::Payload,
        max_duration_secs: 0,
        win_score: 1500,
        starting_reinforcements: 0,
        bracket: LevelBracket::new(30, 90),
    }
}

/// BattlemasterList894/map1803, build12.1.0.69933; TDB WorldState13846.
pub fn seething_shore() -> BgTemplate {
    BgTemplate {
        id: 894,
        name: "Seething Shore".into(),
        map_id: 1803,
        team_size: 10,
        min_players_per_team: 5,
        objective: BgObjective::ResourceRace,
        max_duration_secs: 0,
        win_score: 1500,
        starting_reinforcements: 0,
        bracket: LevelBracket::new(40, 90),
    }
}

/// BattlemasterList1110/map2656, build12.1.0.69933; TDB WorldState24960.
pub fn deephaul_ravine() -> BgTemplate {
    BgTemplate {
        id: 1110,
        name: "Deephaul Ravine".into(),
        map_id: 2656,
        team_size: 10,
        min_players_per_team: 5,
        objective: BgObjective::Payload,
        max_duration_secs: 0,
        win_score: 1500,
        starting_reinforcements: 0,
        bracket: LevelBracket::new(19, 90),
    }
}

/// All built-in battleground templates.
pub fn all_templates() -> Vec<BgTemplate> {
    vec![
        warsong_gulch(),
        arathi_basin(),
        battle_for_gilneas(),
        twin_peaks(),
        eye_of_the_storm(),
        temple_of_kotmogu(),
        silvershard_mines(),
        seething_shore(),
        deephaul_ravine(),
        alterac_valley(),
        isle_of_conquest(),
    ]
}

/// Find a template by ID.
pub fn template_by_id(id: u32) -> Option<BgTemplate> {
    all_templates().into_iter().find(|t| t.id == id)
}

/// Authored TDB objective objects a match on `map_id` needs loaded and usable.
pub fn objective_object_entries(map_id: u32) -> Vec<u32> {
    if map_id == 30 {
        return alterac_banner_entries();
    }
    if map_id == 628 {
        let mut entries = isle_banner_entries();
        entries.extend(isle_gate_entries());
        entries.extend(isle_keep_door_entries(BgTeam::A));
        entries.extend(isle_keep_door_entries(BgTeam::B));
        return entries;
    }
    if map_id == 2656 {
        return vec![422413];
    }
    if map_id == 1803 {
        return SEETHING_OBJECTS.to_vec();
    }
    if map_id == 998 {
        // TC OnInit dynamically spawns orange, blue, green, purple from TDB templates.
        return KOTMOGU_ORBS.iter().map(|orb| orb.entry).collect();
    }
    if let Some(rules) = node_rules(map_id) {
        return rules.nodes.iter().map(|node| node.entry).collect();
    }
    if let Some(rules) = eye_rules(map_id) {
        return vec![rules.flag_entry];
    }
    if flag_rules(map_id).is_some() {
        return vec![
            crate::protocol::WARSONG_ALLIANCE_FLAG_ENTRY,
            crate::protocol::WARSONG_HORDE_FLAG_ENTRY,
        ];
    }
    Vec::new()
}

/// Queue/entry eligibility includes NPC objective maps without fabricating GO placements.
pub fn has_implemented_objectives(map_id: u32) -> bool {
    map_id == 727 || !objective_object_entries(map_id).is_empty()
}

/// Whether `entry` is an objective object of any queueable battleground map.
pub fn is_objective_entry(entry: u32) -> bool {
    if entry == DEEPHAUL_DROP_ENTRY {
        return true;
    }
    all_templates()
        .iter()
        .any(|template| objective_object_entries(template.map_id).contains(&entry))
}

// -- BG Queue --

/// A player or group queued for a battleground.
#[derive(Debug, Clone, PartialEq)]
pub struct BgQueueEntry {
    /// Leader entity bits (solo player or group leader).
    pub leader: u64,
    /// All member entity bits.
    pub members: Vec<u64>,
    /// Level of each member (index matches `members`).
    pub levels: Vec<u8>,
    /// Shared authored PVPDifficulty range of every member.
    pub bracket: LevelBracket,
    /// Battleground type ID.
    pub bg_id: u32,
    /// Server timestamp when queued.
    pub queued_at: u64,
}

/// BattlemasterList32 is a queue, not a concrete battleground template.
pub const RANDOM_BATTLEGROUND_ID: u32 = 32;

struct QueueRules {
    id: u32,
    name: String,
    bracket_map: u32,
    levels: LevelBracket,
    minimum: u8,
    capacity: u8,
    max_group: u8,
}

fn queue_rules(id: u32) -> Option<QueueRules> {
    if id == RANDOM_BATTLEGROUND_ID {
        // Authenticated70448 BML32 and first XMap1109; TC Handler uses MapIDs.front().
        return Some(QueueRules {
            id,
            name: "Random Battleground".into(),
            bracket_map: 566,
            levels: LevelBracket::new(10, 90),
            minimum: 5,
            capacity: 10,
            max_group: 5,
        });
    }
    let template = template_by_id(id)?;
    Some(QueueRules {
        id,
        name: template.name,
        bracket_map: template.map_id,
        levels: template.bracket,
        minimum: template.min_players_per_team,
        capacity: template.team_size,
        max_group: template.team_size,
    })
}

/// Queue label, including multi-map queues that have no instance template.
pub fn queue_name(id: u32) -> Option<String> {
    queue_rules(id).map(|rules| rules.name)
}

/// Maximum party size allowed by the requested queue.
pub fn queue_group_limit(id: u32) -> Option<u8> {
    queue_rules(id).map(|rules| rules.max_group)
}

/// Why joining the BG queue failed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BgQueueError {
    /// Player already in the queue.
    AlreadyQueued,
    /// Unknown battleground ID.
    UnknownBg,
    /// Player level outside the BG's bracket.
    OutOfBracket,
    /// Group has no members.
    EmptyGroup,
    /// Group members and level entries do not align.
    LevelCountMismatch,
    /// Group listed the same player more than once.
    DuplicatePlayer,
    /// Party exceeds the requested queue's group limit.
    GroupTooLarge,
    /// Player has an active deserter penalty.
    Deserter,
}

/// A successful battleground match — two teams ready to enter.
#[derive(Debug, Clone, PartialEq)]
pub struct BgMatch {
    pub bg_id: u32,
    pub bracket: LevelBracket,
    pub team_a: Vec<u64>,
    pub team_b: Vec<u64>,
}

/// Battleground matchmaking queue.
///
/// Players join for a specific BG type. Matchmaking fills two teams from
/// queued entries, ignoring roles (unlike dungeon finder). Players are
/// assigned to teams in queue order until both teams reach `team_size`.
#[derive(Debug, Clone, Default)]
pub struct BgQueue {
    entries: Vec<BgQueueEntry>,
}

impl BgQueue {
    /// Queue a solo player.
    pub fn join_solo(
        &mut self,
        player: u64,
        level: u8,
        bg_id: u32,
        now: u64,
    ) -> Result<(), BgQueueError> {
        self.join_group(player, vec![player], vec![level], bg_id, now)
    }

    /// Queue a solo player, checking deserter status.
    pub fn join_solo_checked(
        &mut self,
        player: u64,
        level: u8,
        bg_id: u32,
        now: u64,
        deserters: &BgDeserterTracker,
    ) -> Result<(), BgQueueError> {
        if deserters.is_deserter(player, now) {
            return Err(BgQueueError::Deserter);
        }
        self.join_solo(player, level, bg_id, now)
    }

    /// Queue a group of players.
    pub fn join_group(
        &mut self,
        leader: u64,
        members: Vec<u64>,
        levels: Vec<u8>,
        bg_id: u32,
        now: u64,
    ) -> Result<(), BgQueueError> {
        if members.is_empty() {
            return Err(BgQueueError::EmptyGroup);
        }
        if members.len() != levels.len() {
            return Err(BgQueueError::LevelCountMismatch);
        }
        let rules = queue_rules(bg_id).ok_or(BgQueueError::UnknownBg)?;
        if bg_id == RANDOM_BATTLEGROUND_ID && members.len() > usize::from(rules.max_group) {
            return Err(BgQueueError::GroupTooLarge);
        }
        let bracket =
            source_level_bracket(rules.bracket_map, levels[0]).ok_or(BgQueueError::OutOfBracket)?;
        let same_bracket = levels.iter().all(|&level| {
            rules.levels.contains(level)
                && source_level_bracket(rules.bracket_map, level) == Some(bracket)
        });
        if !same_bracket {
            return Err(BgQueueError::OutOfBracket);
        }
        if has_duplicate_members(&members) {
            return Err(BgQueueError::DuplicatePlayer);
        }
        if members.iter().any(|&member| self.is_queued(member)) {
            return Err(BgQueueError::AlreadyQueued);
        }
        self.entries.push(BgQueueEntry {
            leader,
            members,
            levels,
            bracket,
            bg_id,
            queued_at: now,
        });
        Ok(())
    }

    /// Remove a player/group from the queue.
    pub fn leave(&mut self, leader: u64) -> bool {
        let had = self.entries.iter().any(|e| e.leader == leader);
        self.entries.retain(|e| e.leader != leader);
        had
    }

    /// Whether a player is already queued (as leader or member).
    pub fn is_queued(&self, player: u64) -> bool {
        self.entries.iter().any(|e| e.members.contains(&player))
    }

    /// Actual admission level retained for an individual queued member.
    pub fn player_level(&self, player: u64) -> Option<u8> {
        self.entries.iter().find_map(|entry| {
            entry
                .members
                .iter()
                .position(|&member| member == player)
                .map(|index| entry.levels[index])
        })
    }

    /// Requested queues in first-entry order, for event-driven matchmaking updates.
    pub fn queued_ids(&self) -> Vec<u32> {
        let mut seen = std::collections::BTreeSet::new();
        self.entries
            .iter()
            .filter(|entry| seen.insert(entry.bg_id))
            .map(|entry| entry.bg_id)
            .collect()
    }

    /// Fill only whole groups fitting the smaller team of an existing concrete copy.
    /// BG_BACKFILL_WHOLE_PARTY_SMALLER_TEAM: no TC one-player-at-parity limiter.
    pub fn fill_existing(
        &mut self,
        queue_id: u32,
        instance: &mut BgInstance,
        template: &BgTemplate,
    ) -> Vec<u64> {
        if instance.phase == BgPhase::Ended {
            return Vec::new();
        }
        let candidates = self.existing_copy_candidates(queue_id, template, instance.bracket);
        let (team_a, team_b, used) = fill_teams(
            &self.entries,
            &candidates,
            usize::from(template.team_size),
            instance.team_a_players.clone(),
            instance.team_b_players.clone(),
        );
        let invited = used
            .iter()
            .flat_map(|&index| self.entries[index].members.iter().copied())
            .collect();
        remove_indices(&mut self.entries, &used);
        instance.team_a_players = team_a;
        instance.team_b_players = team_b;
        invited
    }

    fn existing_copy_candidates(
        &self,
        queue_id: u32,
        template: &BgTemplate,
        bracket: LevelBracket,
    ) -> Vec<usize> {
        self.entries
            .iter()
            .enumerate()
            .filter(|(_, entry)| {
                entry.bg_id == queue_id
                    && entry.levels.iter().all(|&level| {
                        template.bracket.contains(level)
                            && source_level_bracket(template.map_id, level) == Some(bracket)
                    })
            })
            .map(|(index, _)| index)
            .collect()
    }

    /// Number of entries in the queue.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Try to form a match for a given BG type.
    ///
    /// Collects queued players for the BG until both teams reach
    /// `min_players_per_team`. Players are assigned alternately to balance
    /// team sizes. Returns `None` if not enough players.
    pub fn try_match(&mut self, bg_id: u32) -> Option<BgMatch> {
        let rules = queue_rules(bg_id)?;
        let mut seen = std::collections::HashSet::new();
        let brackets: Vec<_> = self
            .entries
            .iter()
            .filter(|entry| entry.bg_id == bg_id && seen.insert(entry.bracket))
            .map(|entry| entry.bracket)
            .collect();
        for bracket in brackets {
            if let Some(matched) = self.try_match_bracket(&rules, bracket) {
                return Some(matched);
            }
        }
        None
    }

    fn try_match_bracket(&mut self, rules: &QueueRules, bracket: LevelBracket) -> Option<BgMatch> {
        let candidate_indices = self.candidates_for(rules.id, bracket);
        let total_players: usize = candidate_indices
            .iter()
            .map(|&i| self.entries[i].members.len())
            .sum();
        let min = usize::from(rules.minimum);
        if total_players < min * 2 {
            return None;
        }

        let (team_a, team_b, used) = fill_teams(
            &self.entries,
            &candidate_indices,
            usize::from(rules.capacity),
            Vec::new(),
            Vec::new(),
        );
        if team_a.len() < min || team_b.len() < min {
            return None;
        }

        remove_indices(&mut self.entries, &used);
        Some(BgMatch {
            bg_id: rules.id,
            bracket,
            team_a,
            team_b,
        })
    }

    fn candidates_for(&self, bg_id: u32, bracket: LevelBracket) -> Vec<usize> {
        self.entries
            .iter()
            .enumerate()
            .filter(|(_, e)| e.bg_id == bg_id && e.bracket == bracket)
            .map(|(i, _)| i)
            .collect()
    }
}

fn has_duplicate_members(members: &[u64]) -> bool {
    let mut seen = std::collections::BTreeSet::new();
    members.iter().any(|member| !seen.insert(*member))
}

/// Assign queued entries alternately to two teams up to `target` size each.
fn fill_teams(
    entries: &[BgQueueEntry],
    candidates: &[usize],
    target: usize,
    mut team_a: Vec<u64>,
    mut team_b: Vec<u64>,
) -> (Vec<u64>, Vec<u64>, Vec<usize>) {
    let mut used = Vec::new();

    for &idx in candidates {
        let members = &entries[idx].members;
        let smaller = if team_a.len() <= team_b.len() {
            &mut team_a
        } else {
            &mut team_b
        };
        if smaller.len() + members.len() <= target {
            smaller.extend(members);
            used.push(idx);
        }
        if team_a.len() >= target && team_b.len() >= target {
            break;
        }
    }
    (team_a, team_b, used)
}

fn remove_indices<T>(vec: &mut Vec<T>, indices: &[usize]) {
    let mut sorted = indices.to_vec();
    sorted.sort_unstable();
    for idx in sorted.into_iter().rev() {
        vec.remove(idx);
    }
}

// -- BG Instance State Machine --

/// Which team a player belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BgTeam {
    A,
    B,
}

/// Phase of a battleground instance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BgPhase {
    /// Players are entering; waiting for min_players_per_team on each side.
    Waiting,
    /// Match is live.
    InProgress,
    /// Match has ended (winner determined or time expired).
    Ended,
}

/// Outcome of a finished battleground.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BgOutcome {
    /// One team won by reaching win_score or depleting enemy reinforcements.
    Winner(BgTeam),
    /// Time expired with no winner — draw.
    Draw,
}

/// Per-team score in a battleground instance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TeamScore {
    pub score: u32,
    pub reinforcements: u32,
}

/// A live battleground instance.
#[derive(Debug, Clone, PartialEq)]
pub struct BgInstance {
    pub bg_id: u32,
    pub bracket: LevelBracket,
    pub phase: BgPhase,
    pub team_a_players: Vec<u64>,
    pub team_b_players: Vec<u64>,
    pub score_a: TeamScore,
    pub score_b: TeamScore,
    pub elapsed_secs: u32,
    pub outcome: Option<BgOutcome>,
}

impl BgInstance {
    /// Create a new instance from a match result and template.
    pub fn from_match(bg_match: &BgMatch, tmpl: &BgTemplate) -> Self {
        Self {
            bg_id: bg_match.bg_id,
            bracket: bg_match.bracket,
            phase: BgPhase::Waiting,
            team_a_players: bg_match.team_a.clone(),
            team_b_players: bg_match.team_b.clone(),
            score_a: TeamScore {
                score: 0,
                reinforcements: tmpl.starting_reinforcements,
            },
            score_b: TeamScore {
                score: 0,
                reinforcements: tmpl.starting_reinforcements,
            },
            elapsed_secs: 0,
            outcome: None,
        }
    }

    /// Transition from Waiting to InProgress. Returns false if already started/ended.
    pub fn start(&mut self) -> bool {
        if self.phase != BgPhase::Waiting {
            return false;
        }
        self.phase = BgPhase::InProgress;
        true
    }

    /// Add score points for a team (flag captures, resource ticks).
    /// Checks win condition against the template's `win_score`.
    pub fn add_score(&mut self, team: BgTeam, points: u32, tmpl: &BgTemplate) {
        if self.phase != BgPhase::InProgress {
            return;
        }
        let score = match team {
            BgTeam::A => &mut self.score_a,
            BgTeam::B => &mut self.score_b,
        };
        score.score += points;
        if tmpl.win_score > 0 && score.score >= tmpl.win_score {
            score.score = tmpl.win_score;
            self.end(BgOutcome::Winner(team));
        }
    }

    /// Deduct reinforcements from a team. Triggers loss at zero.
    pub fn deduct_reinforcements(&mut self, team: BgTeam, amount: u32) {
        if self.phase != BgPhase::InProgress {
            return;
        }
        let score = match team {
            BgTeam::A => &mut self.score_a,
            BgTeam::B => &mut self.score_b,
        };
        score.reinforcements = score.reinforcements.saturating_sub(amount);
        if score.reinforcements == 0 {
            let winner = match team {
                BgTeam::A => BgTeam::B,
                BgTeam::B => BgTeam::A,
            };
            self.end(BgOutcome::Winner(winner));
        }
    }

    /// Advance elapsed time. Ends the match as a draw if max_duration exceeded.
    pub fn tick(&mut self, delta_secs: u32, tmpl: &BgTemplate) {
        if self.phase != BgPhase::InProgress {
            return;
        }
        self.elapsed_secs += delta_secs;
        if tmpl.max_duration_secs > 0 && self.elapsed_secs >= tmpl.max_duration_secs {
            self.end(BgOutcome::Draw);
        }
    }

    pub fn end(&mut self, outcome: BgOutcome) {
        self.phase = BgPhase::Ended;
        self.outcome = Some(outcome);
    }
}

#[path = "battleground_objectives.rs"]
mod objectives;
pub use objectives::*;
#[path = "battleground_alterac.rs"]
mod alterac;
pub use alterac::*;
#[cfg(test)]
#[path = "battleground_alterac_tests.rs"]
mod alterac_tests;
#[path = "battleground_isle.rs"]
mod isle;
pub use isle::*;
#[cfg(test)]
#[path = "battleground_isle_tests.rs"]
mod isle_tests;

#[path = "battleground_nodes.rs"]
mod nodes;
pub use nodes::*;
#[path = "battleground_flags.rs"]
mod flags;
pub use flags::*;
#[path = "battleground_columns.rs"]
mod columns;
pub use columns::*;
#[path = "battleground_eye.rs"]
mod eye;
pub use eye::*;
#[path = "battleground_kotmogu.rs"]
mod kotmogu;
pub use kotmogu::*;
#[path = "battleground_silvershard.rs"]
mod silvershard;
pub use silvershard::*;

#[path = "battleground_seething.rs"]
mod seething;
pub use seething::*;

#[cfg(test)]
#[path = "battleground_seething_tests.rs"]
mod seething_tests;

#[path = "battleground_deephaul.rs"]
mod deephaul;
#[path = "battleground_motion.rs"]
mod motion;
pub use deephaul::*;

#[cfg(test)]
#[path = "battleground_deephaul_tests.rs"]
mod deephaul_tests;

// -- Scoring & Rewards --

/// Honor currency ID (matches `currency.rs` ALL_CURRENCIES).
pub const HONOR_CURRENCY_ID: u32 = 1;

/// Base honor awarded to the winning team.
const WINNER_HONOR: u32 = 150;
/// Base honor awarded to the losing team.
const LOSER_HONOR: u32 = 50;
/// Honor for a draw (both teams).
const DRAW_HONOR: u32 = 75;
/// Marks awarded to the winning team.
const WINNER_MARKS: u32 = 3;
/// Marks awarded to the losing team.
const LOSER_MARKS: u32 = 1;
/// Marks for a draw.
const DRAW_MARKS: u32 = 2;

/// Rewards for a single player after a battleground ends.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BgReward {
    pub honor: u32,
    pub marks: u32,
}

/// Per-player reward list for a completed battleground.
#[derive(Debug, Clone, PartialEq)]
pub struct BgRewardDistribution {
    pub rewards: Vec<(u64, BgReward)>,
}

/// Compute rewards for all players in a finished battleground.
///
/// Winners get more honor and marks than losers. Draws get an
/// intermediate amount. Returns `None` if the match hasn't ended.
pub fn distribute_rewards(instance: &BgInstance) -> Option<BgRewardDistribution> {
    let outcome = instance.outcome?;
    if instance.phase != BgPhase::Ended {
        return None;
    }

    let (reward_a, reward_b) = team_rewards(outcome);
    let mut rewards = team_reward_entries(&instance.team_a_players, reward_a);
    rewards.extend(team_reward_entries(&instance.team_b_players, reward_b));
    Some(BgRewardDistribution { rewards })
}

fn team_rewards(outcome: BgOutcome) -> (BgReward, BgReward) {
    match outcome {
        BgOutcome::Winner(BgTeam::A) => (winner_reward(), loser_reward()),
        BgOutcome::Winner(BgTeam::B) => (loser_reward(), winner_reward()),
        BgOutcome::Draw => (draw_reward(), draw_reward()),
    }
}

fn winner_reward() -> BgReward {
    BgReward {
        honor: WINNER_HONOR,
        marks: WINNER_MARKS,
    }
}

fn loser_reward() -> BgReward {
    BgReward {
        honor: LOSER_HONOR,
        marks: LOSER_MARKS,
    }
}

fn draw_reward() -> BgReward {
    BgReward {
        honor: DRAW_HONOR,
        marks: DRAW_MARKS,
    }
}

fn team_reward_entries(players: &[u64], reward: BgReward) -> Vec<(u64, BgReward)> {
    players
        .iter()
        .copied()
        .map(|player| (player, reward))
        .collect()
}

// -- Deserter Penalty --

/// BG deserter debuff duration in seconds (15 minutes).
pub const BG_DESERTER_DURATION: u64 = 900;

/// Tracks BG deserter penalties for players who leave matches early.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct BgDeserterTracker {
    /// (player_entity, expires_at_timestamp) pairs.
    entries: Vec<(u64, u64)>,
}

impl BgDeserterTracker {
    /// Apply a deserter penalty to a player.
    pub fn apply(&mut self, player: u64, now: u64) {
        let expires = now + BG_DESERTER_DURATION;
        if let Some(entry) = self.entries.iter_mut().find(|(p, _)| *p == player) {
            entry.1 = expires;
        } else {
            self.entries.push((player, expires));
        }
    }

    /// Whether a player has an active deserter penalty.
    pub fn is_deserter(&self, player: u64, now: u64) -> bool {
        self.entries
            .iter()
            .any(|(p, exp)| *p == player && now < *exp)
    }

    /// Remaining deserter time in seconds (0 if not active).
    pub fn remaining(&self, player: u64, now: u64) -> u64 {
        self.entries
            .iter()
            .find(|(p, _)| *p == player)
            .map_or(0, |(_, exp)| exp.saturating_sub(now))
    }

    /// Remove expired penalties.
    pub fn cleanup(&mut self, now: u64) {
        self.entries.retain(|(_, exp)| now < *exp);
    }
}

/// Process a player leaving an in-progress BG. Applies deserter and
/// removes them from the instance. Returns false if player not found.
pub fn leave_bg(
    instance: &mut BgInstance,
    deserters: &mut BgDeserterTracker,
    player: u64,
    now: u64,
) -> bool {
    if instance.phase != BgPhase::InProgress {
        return false;
    }
    let in_a = instance.team_a_players.contains(&player);
    let in_b = instance.team_b_players.contains(&player);
    if !in_a && !in_b {
        return false;
    }
    if in_a {
        instance.team_a_players.retain(|&p| p != player);
    } else {
        instance.team_b_players.retain(|&p| p != player);
    }
    deserters.apply(player, now);
    true
}

#[cfg(test)]
#[path = "battleground_tests.rs"]
mod tests;
