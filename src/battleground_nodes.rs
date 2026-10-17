//! Retail type 42 capture points. Values: TDB1210 templates and TC a352b1fa.
use super::{BgInstance, BgTeam, BgTemplate, NodeOwner};

#[derive(Debug, Clone, Copy)]
pub struct NodeSpec {
    pub entry: u32,
    pub name: &'static str,
    pub poi: u32,
    pub alliance_widget: u32,
    pub horde_widget: u32,
}

#[derive(Debug)]
pub struct NodeRules {
    pub map_id: u32,
    pub starts: [u32; 2],
    pub tick_ms: u64,
    pub points: &'static [u32],
    pub assault_stat: u32,
    pub defense_stat: u32,
    pub nodes: &'static [NodeSpec],
}

// CapturePoint.CaptureTime (Data0) is 60000 for all eight source templates.
const CAPTURE_MS: u64 = 60_000;
const ARATHI: NodeRules = NodeRules {
    map_id: 2107,
    starts: [7056, 7055],
    tick_ms: 2000,
    points: &[0, 2, 3, 4, 7, 60],
    assault_stat: 926,
    defense_stat: 927,
    nodes: &[
        NodeSpec {
            entry: 227420,
            name: "Stables",
            poi: 5834,
            alliance_widget: 17322,
            horde_widget: 17331,
        },
        NodeSpec {
            entry: 227522,
            name: "Blacksmith",
            poi: 8799,
            alliance_widget: 17324,
            horde_widget: 17327,
        },
        NodeSpec {
            entry: 227536,
            name: "Farm",
            poi: 8808,
            alliance_widget: 17325,
            horde_widget: 17328,
        },
        NodeSpec {
            entry: 227538,
            name: "Gold Mine",
            poi: 8809,
            alliance_widget: 17323,
            horde_widget: 17329,
        },
        NodeSpec {
            entry: 227544,
            name: "Lumber Mill",
            poi: 8805,
            alliance_widget: 17326,
            horde_widget: 17330,
        },
    ],
};
const GILNEAS: NodeRules = NodeRules {
    map_id: 761,
    starts: [1798, 1799],
    tick_ms: 1000,
    points: &[0, 1, 3, 30],
    assault_stat: 370,
    defense_stat: 371,
    nodes: &[
        NodeSpec {
            entry: 228050,
            name: "Lighthouse",
            poi: 8851,
            alliance_widget: 17356,
            horde_widget: 17355,
        },
        NodeSpec {
            entry: 228052,
            name: "Waterworks",
            poi: 8852,
            alliance_widget: 17357,
            horde_widget: 17358,
        },
        NodeSpec {
            entry: 228053,
            name: "Mines",
            poi: 8853,
            alliance_widget: 17359,
            horde_widget: 17360,
        },
    ],
};

pub fn node_rules(map_id: u32) -> Option<&'static NodeRules> {
    match map_id {
        2107 => Some(&ARATHI),
        761 => Some(&GILNEAS),
        _ => None,
    }
}

#[derive(Debug)]
struct NodeState {
    owner: NodeOwner,
    last_owner: Option<BgTeam>,
    capture_left_ms: u64,
}

#[derive(Debug)]
pub struct NodeBattle {
    rules: &'static NodeRules,
    nodes: Vec<NodeState>,
    tick_left_ms: u64,
    // Unclamped points emitted by the source clock; external score changes do not enter it.
    resource_points: [u32; 2],
}

impl NodeBattle {
    pub fn new(rules: &'static NodeRules) -> Self {
        Self {
            rules,
            nodes: rules
                .nodes
                .iter()
                .map(|_| NodeState {
                    owner: NodeOwner::Neutral,
                    last_owner: None,
                    capture_left_ms: 0,
                })
                .collect(),
            tick_left_ms: rules.tick_ms,
            resource_points: [0; 2],
        }
    }

    /// Returns the source PvP stat credited by an actual assault or defense.
    pub fn assault(&mut self, entry: u32, team: BgTeam) -> Result<u32, &'static str> {
        let index = self
            .rules
            .nodes
            .iter()
            .position(|node| node.entry == entry)
            .ok_or("object is not an authored capture point for this match")?;
        let node = &mut self.nodes[index];
        if matches!(node.owner, NodeOwner::Owned(owner) | NodeOwner::Contested(owner) if owner == team)
        {
            return Err("capture point already belongs to this team");
        }
        if node.last_owner == Some(team) {
            node.owner = NodeOwner::Owned(team);
            node.capture_left_ms = 0;
            return Ok(self.rules.defense_stat);
        }
        node.owner = NodeOwner::Contested(team);
        node.capture_left_ms = CAPTURE_MS;
        Ok(self.rules.assault_stat)
    }

    /// Process capture and resource deadlines in order, including long frames.
    pub fn advance(&mut self, mut delta_ms: u64, instance: &mut BgInstance, template: &BgTemplate) {
        while delta_ms > 0 && instance.phase == super::BgPhase::InProgress {
            let capture_deadline = self
                .nodes
                .iter()
                .filter(|node| matches!(node.owner, NodeOwner::Contested(_)))
                .map(|node| node.capture_left_ms)
                .min()
                .unwrap_or(u64::MAX);
            let step = delta_ms.min(self.tick_left_ms).min(capture_deadline);
            self.advance_captures(step);
            self.tick_left_ms -= step;
            delta_ms -= step;
            if self.tick_left_ms == 0 {
                self.award_resources(instance, template);
                self.tick_left_ms = self.rules.tick_ms;
            }
        }
    }

    fn advance_captures(&mut self, delta_ms: u64) {
        for node in &mut self.nodes {
            let NodeOwner::Contested(team) = node.owner else {
                continue;
            };
            node.capture_left_ms -= delta_ms;
            if node.capture_left_ms == 0 {
                node.owner = NodeOwner::Owned(team);
                node.last_owner = Some(team);
            }
        }
    }

    pub fn earned_resource_points(&self) -> [u32; 2] {
        self.resource_points
    }

    fn award_resources(&mut self, instance: &mut BgInstance, template: &BgTemplate) {
        for (index, team) in [BgTeam::A, BgTeam::B].into_iter().enumerate() {
            let count = self.owned_count(team);
            let points = self.rules.points[count];
            self.resource_points[index] += points;
            instance.add_score(team, points, template);
        }
    }

    fn owned_count(&self, team: BgTeam) -> usize {
        self.nodes
            .iter()
            .filter(|node| node.owner == NodeOwner::Owned(team))
            .count()
    }

    pub fn world_states(&self, instance: &BgInstance) -> Vec<(u32, i32)> {
        let mut values = vec![
            (1776, instance.score_a.score as i32),
            (1777, instance.score_b.score as i32),
            (1779, self.owned_count(BgTeam::A) as i32),
            (1778, self.owned_count(BgTeam::B) as i32),
            (1780, 1500),
        ];
        for (spec, node) in self.rules.nodes.iter().zip(&self.nodes) {
            values.push((spec.poi, capture_point_state(node.owner)));
            values.push((spec.alliance_widget, control_state(node.owner, BgTeam::A)));
            values.push((spec.horde_widget, control_state(node.owner, BgTeam::B)));
        }
        values
    }
}

// TC BattlegroundPackets.h BattlegroundCapturePointState; widget values are
// explicitly 0 uncontrolled, 1 partial, 2 captured in both pinned map scripts.
pub fn capture_point_state(owner: NodeOwner) -> i32 {
    match owner {
        NodeOwner::Neutral => 1,
        NodeOwner::Contested(BgTeam::B) => 2,
        NodeOwner::Contested(BgTeam::A) => 3,
        NodeOwner::Owned(BgTeam::B) => 4,
        NodeOwner::Owned(BgTeam::A) => 5,
    }
}
fn control_state(owner: NodeOwner, team: BgTeam) -> i32 {
    match owner {
        NodeOwner::Owned(owner) if owner == team => 2,
        NodeOwner::Contested(owner) if owner == team => 1,
        _ => 0,
    }
}

#[cfg(test)]
#[path = "battleground_tests/timed_nodes.rs"]
mod tests;
