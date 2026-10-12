//! TC a352b1fa Deephaul carts/crystal; TDB1210 paths and templates415326/415330/422413.
use super::motion::{distance, follow_linear_path};
use super::{BgInstance, BgOutcome, BgPhase, BgTeam, BgTemplate, CartEvent, CartPaths, FlagSite};

pub const DEEPHAUL_STARTS: [u32; 2] = [100071, 100070];
pub const DEEPHAUL_FLAG_ENTRY: u32 = 422413;
pub const DEEPHAUL_DROP_ENTRY: u32 = 422412;
pub const DEEPHAUL_CAPTURE_TRIGGERS: [[f32; 3]; 2] =
    [[3919.47, -3120.23, 240.846], [4207.9, -2777.05, 239.817]];
pub const DEEPHAUL_FLAG_STAT: u32 = 1020;
pub const DEEPHAUL_CONTROL_STAT: u32 = 1021;
const TRIGGER_RADIUS: f32 = 5.0;
const CONTROL_RADIUS: f32 = 30.0;
const CONTROL_RATE: i32 = 100 / 50;
const HEARTBEAT_MS: u64 = 1000;
const TICK_MS: u64 = 2000;
const TICK_POINTS: u32 = 3;
const DELIVERY_POINTS: u32 = 100;
const CART_RESPAWN_MS: u64 = 2000;
const FLAG_ENABLE_MS: u64 = 15_000;
const FLAG_EXPIRE_MS: u64 = 10_000;
const FLAG_RESPAWN_MS: u64 = 30_000;
const FLAG_POINTS: u32 = 100;
// Both NPC templates: speed_walk1.4; TC base walk2.5yd/s, exact linear Walk paths.
pub const DEEPHAUL_CART_SPEED: f32 = 2.5 * 1.4;

#[derive(Debug)]
pub struct DeephaulCartDefinition {
    pub name: &'static str,
    pub entry: u32,
    pub spawn: [f32; 3],
    pub orientation: f32,
    pub path: u32,
    pub control_entry: u32,
    pub world_states: [u32; 2],
    initial_value: i32,
}

pub const DEEPHAUL_CARTS: [DeephaulCartDefinition; 2] = [
    DeephaulCartDefinition {
        name: "East Cart",
        entry: 214690,
        spawn: [3875.0, -3150.0, 240.2913],
        orientation: 0.0,
        path: 21469000,
        control_entry: 415326,
        world_states: [25415, 25414],
        initial_value: 100,
    },
    DeephaulCartDefinition {
        name: "West Cart",
        entry: 217346,
        spawn: [4250.3647, -2751.066, 239.46973],
        orientation: std::f32::consts::PI,
        path: 21734600,
        control_entry: 415330,
        world_states: [25421, 25420],
        initial_value: 0,
    },
];

#[derive(Debug)]
struct Cart {
    position: [f32; 3],
    next_node: usize,
    value: i32,
    displayed_owner: BgTeam,
    arrived: bool,
}

impl Cart {
    fn new(definition: &DeephaulCartDefinition) -> Self {
        Self {
            position: definition.spawn,
            next_node: 0,
            value: definition.initial_value,
            displayed_owner: if definition.initial_value == 100 {
                BgTeam::A
            } else {
                BgTeam::B
            },
            arrived: false,
        }
    }

    fn owner(&self) -> Option<BgTeam> {
        match self.value {
            value if value > 50 => Some(BgTeam::A),
            value if value < 50 => Some(BgTeam::B),
            _ => None,
        }
    }

    fn heartbeat(&mut self, players: &[(u64, BgTeam, [f32; 3])]) -> Vec<CartEvent> {
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
        self.value = (self.value + direction * CONTROL_RATE).clamp(0, 100);
        let Some(owner) = self.owner().filter(|owner| Some(*owner) != previous) else {
            return Vec::new();
        };
        // Neutral events are not handled by this TC script; displayed states retain control.
        self.displayed_owner = owner;
        inside
            .into_iter()
            .filter(|(_, team, _)| *team == owner)
            .map(|(id, _, _)| CartEvent::Controlled(*id))
            .collect()
    }
}

#[derive(Debug)]
enum Crystal {
    Disabled { left_ms: u64 },
    Home,
    Taken { carrier: u64, team: BgTeam },
    Dropped { position: [f32; 3], left_ms: u64 },
    Respawning { left_ms: u64 },
}

impl Crystal {
    fn timer(&self) -> Option<u64> {
        match self {
            Self::Disabled { left_ms }
            | Self::Dropped { left_ms, .. }
            | Self::Respawning { left_ms } => Some(*left_ms),
            _ => None,
        }
    }

    fn elapse(&mut self, step: u64) {
        let (Self::Disabled { left_ms }
        | Self::Dropped { left_ms, .. }
        | Self::Respawning { left_ms }) = self
        else {
            return;
        };
        *left_ms -= step;
        if *left_ms == 0 {
            *self = Self::Home;
        }
    }
}

#[derive(Debug)]
pub struct DeephaulBattle {
    carts: [Cart; 2],
    respawn_left_ms: Option<u64>,
    heartbeat_left_ms: u64,
    tick_left_ms: u64,
    crystal: Crystal,
    captured: [bool; 2],
}

impl Default for DeephaulBattle {
    fn default() -> Self {
        Self::new()
    }
}

impl DeephaulBattle {
    pub fn new() -> Self {
        Self {
            carts: std::array::from_fn(|index| Cart::new(&DEEPHAUL_CARTS[index])),
            respawn_left_ms: None,
            heartbeat_left_ms: HEARTBEAT_MS,
            tick_left_ms: TICK_MS,
            crystal: Crystal::Disabled {
                left_ms: FLAG_ENABLE_MS,
            },
            captured: [false; 2],
        }
    }

    pub fn sites(&self) -> [Option<[f32; 3]>; 2] {
        std::array::from_fn(|index| {
            self.respawn_left_ms
                .is_none()
                .then_some(self.carts[index].position)
        })
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
            let step = delta_ms
                .min(self.heartbeat_left_ms)
                .min(self.tick_left_ms)
                .min(self.respawn_left_ms.unwrap_or(u64::MAX))
                .min(self.crystal.timer().unwrap_or(u64::MAX));
            delta_ms -= step;
            self.heartbeat_left_ms -= step;
            self.tick_left_ms -= step;
            self.crystal.elapse(step);
            self.advance_carts(step, paths, instance, template, &mut events);
            if instance.phase != BgPhase::InProgress {
                break;
            }
            if self.heartbeat_left_ms == 0 {
                if self.respawn_left_ms.is_none() {
                    for cart in &mut self.carts {
                        events.extend(cart.heartbeat(players));
                    }
                }
                self.heartbeat_left_ms = HEARTBEAT_MS;
            }
            if self.tick_left_ms == 0 {
                self.score_tick(instance, template);
                self.tick_left_ms = TICK_MS;
            }
        }
        events
    }

    fn advance_carts(
        &mut self,
        step: u64,
        paths: &CartPaths,
        instance: &mut BgInstance,
        template: &BgTemplate,
        events: &mut Vec<CartEvent>,
    ) {
        if let Some(left) = &mut self.respawn_left_ms {
            *left -= step;
            if *left == 0 {
                self.carts = std::array::from_fn(|index| Cart::new(&DEEPHAUL_CARTS[index]));
                self.respawn_left_ms = None;
            }
            return;
        }
        for (cart, definition) in self.carts.iter_mut().zip(&DEEPHAUL_CARTS) {
            if cart.arrived {
                continue;
            }
            let mut distance_left = step as f32 / 1000.0 * DEEPHAUL_CART_SPEED;
            cart.arrived = follow_linear_path(
                &mut cart.position,
                &mut cart.next_node,
                &paths[&definition.path],
                &mut distance_left,
            );
        }
        if self.carts.iter().all(|cart| cart.arrived) {
            self.deliver_carts(instance, template, events);
        }
    }

    fn deliver_carts(
        &mut self,
        instance: &mut BgInstance,
        template: &BgTemplate,
        events: &mut Vec<CartEvent>,
    ) {
        for (index, cart) in self.carts.iter().enumerate() {
            let owner = cart.owner();
            events.push(CartEvent::Delivered {
                cart: index,
                team: owner,
                position: cart.position,
            });
            if let Some(team) = owner {
                self.captured[if team == BgTeam::A { 0 } else { 1 }] = true;
                let score = if team == BgTeam::A {
                    &mut instance.score_a.score
                } else {
                    &mut instance.score_b.score
                };
                *score = (*score + DELIVERY_POINTS).min(template.win_score);
            }
        }
        self.respawn_left_ms = Some(CART_RESPAWN_MS);
        check_winner(instance, template);
    }

    fn score_tick(&self, instance: &mut BgInstance, template: &BgTemplate) {
        let count = |team| {
            if self.respawn_left_ms.is_some() {
                0
            } else {
                self.carts
                    .iter()
                    .filter(|cart| cart.displayed_owner == team)
                    .count() as u32
            }
        };
        instance.score_a.score =
            (instance.score_a.score + count(BgTeam::A) * TICK_POINTS).min(template.win_score);
        instance.score_b.score =
            (instance.score_b.score + count(BgTeam::B) * TICK_POINTS).min(template.win_score);
        check_winner(instance, template);
    }

    pub fn pickup(&mut self, team: BgTeam, carrier: u64) -> bool {
        if !matches!(self.crystal, Crystal::Home | Crystal::Dropped { .. }) {
            return false;
        }
        self.crystal = Crystal::Taken { carrier, team };
        true
    }

    pub fn carrier(&self) -> Option<u64> {
        match self.crystal {
            Crystal::Taken { carrier, .. } => Some(carrier),
            _ => None,
        }
    }

    pub fn drop_flag(&mut self, position: [f32; 3]) {
        if self.carrier().is_some() {
            self.crystal = Crystal::Dropped {
                position,
                left_ms: FLAG_EXPIRE_MS,
            };
        }
    }

    pub fn capture(
        &mut self,
        carrier: u64,
        team: BgTeam,
        position: [f32; 3],
        instance: &mut BgInstance,
        template: &BgTemplate,
    ) -> bool {
        if instance.phase != BgPhase::InProgress
            || !matches!(self.crystal,Crystal::Taken { carrier:id,team:owner } if id==carrier && owner==team)
        {
            return false;
        }
        let index = if team == BgTeam::A { 0 } else { 1 };
        if distance(position, DEEPHAUL_CAPTURE_TRIGGERS[index]) > TRIGGER_RADIUS {
            return false;
        }
        let score = if team == BgTeam::A {
            &mut instance.score_a.score
        } else {
            &mut instance.score_b.score
        };
        *score = (*score + FLAG_POINTS).min(template.win_score);
        // TC checks flag-capture victory on its next scheduled scoring tick.
        self.crystal = Crystal::Respawning {
            left_ms: FLAG_RESPAWN_MS,
        };
        true
    }

    pub fn flag_site(&self) -> Option<FlagSite> {
        match self.crystal {
            Crystal::Home => Some(FlagSite::Home),
            Crystal::Dropped { position, .. } => Some(FlagSite::Dropped(position)),
            _ => None,
        }
    }

    pub fn world_states(&self, instance: &BgInstance) -> Vec<(u32, i32)> {
        let carrier_team = match self.crystal {
            Crystal::Taken { team, .. } => Some(team),
            _ => None,
        };
        let mut values = vec![
            (24958, instance.score_a.score as i32),
            (24959, instance.score_b.score as i32),
            (24960, 1500),
            (
                25412,
                if matches!(self.crystal, Crystal::Disabled { .. }) {
                    0
                } else {
                    3
                },
            ),
            (7852, i32::from(instance.phase != BgPhase::Waiting)),
            (
                1545,
                if carrier_team == Some(BgTeam::A) {
                    2
                } else {
                    1
                },
            ),
            (
                1546,
                if carrier_team == Some(BgTeam::B) {
                    2
                } else {
                    1
                },
            ),
            (6954, i32::from(self.captured[0])),
            (6955, i32::from(self.captured[1])),
        ];
        for (cart, definition) in self.carts.iter().zip(&DEEPHAUL_CARTS) {
            values.extend([
                (
                    definition.world_states[0],
                    i32::from(self.respawn_left_ms.is_none() && cart.displayed_owner == BgTeam::A),
                ),
                (
                    definition.world_states[1],
                    i32::from(self.respawn_left_ms.is_none() && cart.displayed_owner == BgTeam::B),
                ),
            ]);
        }
        values
    }
}

fn check_winner(instance: &mut BgInstance, template: &BgTemplate) {
    let outcome = match (
        instance.score_a.score >= template.win_score,
        instance.score_b.score >= template.win_score,
    ) {
        (true, true) => BgOutcome::Draw,
        (true, false) => BgOutcome::Winner(BgTeam::A),
        (false, true) => BgOutcome::Winner(BgTeam::B),
        (false, false) => return,
    };
    instance.phase = BgPhase::Ended;
    instance.outcome = Some(outcome);
}
