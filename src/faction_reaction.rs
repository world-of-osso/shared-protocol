//! FactionTemplate reaction rules as pure functions over DB2 rows (no Bevy,
//! storage or protocol types), shared by server and client.
//!
//! `reaction` is AzerothCore's template-only `Unit::GetFactionReactionTo`
//! check; player reputation (forced ranks, at-war) is not consulted.

/// AzerothCore `FACTION_TEMPLATE_FLAG_HATES_ALL_EXCEPT_FRIENDS`.
const HOSTILE_BY_DEFAULT: u32 = 0x2000;

/// One `faction_template` row (DB2 FactionTemplate).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FactionTemplateEntry {
    pub id: u32,
    pub faction: u32,
    pub flags: u32,
    pub faction_group: u32,
    pub friend_group: u32,
    pub enemy_group: u32,
    pub enemies: [u32; 8],
    pub friends: [u32; 8],
}

impl FactionTemplateEntry {
    /// AzerothCore `FactionTemplateEntry::IsFriendlyTo`.
    fn is_friendly_to(&self, other: &Self) -> bool {
        if self.faction == other.faction {
            return true;
        }
        if other.faction != 0 {
            if self.enemies.contains(&other.faction) {
                return false;
            }
            if self.friends.contains(&other.faction) {
                return true;
            }
        }
        self.friend_group & other.faction_group != 0 || self.faction_group & other.friend_group != 0
    }

    /// AzerothCore `FactionTemplateEntry::IsHostileTo`.
    fn is_hostile_to(&self, other: &Self) -> bool {
        if other.faction != 0 {
            if self.enemies.contains(&other.faction) {
                return true;
            }
            if self.friends.contains(&other.faction) {
                return false;
            }
        }
        self.enemy_group & other.faction_group != 0
    }
}

/// How one unit regards another. Unfriendly (a reputation rank) needs the
/// reputation model and is not produced yet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Reaction {
    Hostile,
    Neutral,
    Friendly,
}

/// One side of an attack/assist check. `template: None` = no or unknown
/// template (neutral to everyone).
#[derive(Debug, Clone, Copy)]
pub struct Unit<'a> {
    pub template: Option<&'a FactionTemplateEntry>,
    pub is_player: bool,
}

/// Reaction of template `own` towards template `other` (template checks of
/// `GetFactionReactionTo`). A missing template on either side is neutral.
pub fn reaction(
    own: Option<&FactionTemplateEntry>,
    other: Option<&FactionTemplateEntry>,
) -> Reaction {
    let (Some(own), Some(other)) = (own, other) else {
        return Reaction::Neutral;
    };
    if own.id == other.id {
        return Reaction::Friendly;
    }
    if own.is_hostile_to(other) {
        Reaction::Hostile
    } else if own.is_friendly_to(other) || other.is_friendly_to(own) {
        Reaction::Friendly
    } else if own.flags & HOSTILE_BY_DEFAULT != 0 {
        Reaction::Hostile
    } else {
        Reaction::Neutral
    }
}

/// AzerothCore `_IsValidAttackTarget`, faction part: never a friendly
/// unit; creature vs creature only when one is hostile to the other.
/// There is no PvP flag or duel model, so player vs player is rejected.
pub fn can_attack(a: Unit, b: Unit) -> bool {
    let forward = reaction(a.template, b.template);
    let backward = reaction(b.template, a.template);
    if forward == Reaction::Friendly || backward == Reaction::Friendly {
        return false;
    }
    match (a.is_player, b.is_player) {
        (true, true) => false,
        (false, false) => forward == Reaction::Hostile || backward == Reaction::Hostile,
        _ => true,
    }
}

/// Assist (friendly spell) target: `a` regards `b` as friendly. Stands in
/// for `_IsValidAssistTarget`'s reaction check plus its PvC rule (creature
/// `type_flags` CAN_ASSIST is not imported).
pub fn can_assist(a: Unit, b: Unit) -> bool {
    reaction(a.template, b.template) == Reaction::Friendly
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `FactionTemplate.csv` rows, wago.tools build 12.1.0.69933.
    fn row(
        id: u32,
        faction: u32,
        flags: u32,
        groups: [u32; 3],
        enemy: u32,
        friend: u32,
    ) -> FactionTemplateEntry {
        let [faction_group, friend_group, enemy_group] = groups;
        FactionTemplateEntry {
            id,
            faction,
            flags,
            faction_group,
            friend_group,
            enemy_group,
            enemies: [enemy, 0, 0, 0, 0, 0, 0, 0],
            friends: [friend, 0, 0, 0, 0, 0, 0, 0],
        }
    }

    fn human() -> FactionTemplateEntry {
        row(1, 1, 72, [3, 2, 12], 0, 0)
    }

    #[test]
    fn retail_templates_regard_a_human_player_as_expected() {
        let human = human();
        let orc = row(2, 2, 72, [5, 4, 10], 0, 0);
        let defias_thug = row(7, 7, 0, [0, 0, 0], 0, 0);
        let stormwind_guard = row(11, 72, 2081, [3, 2, 12], 0, 0);
        let monster = row(14, 14, 0, [8, 0, 1], 0, 0);
        let diseased_wolf = row(32, 29, 16, [0, 0, 0], 28, 0);
        let player = Some(&human);
        assert_eq!(reaction(Some(&defias_thug), player), Reaction::Neutral);
        assert_eq!(reaction(Some(&diseased_wolf), player), Reaction::Neutral);
        assert_eq!(reaction(Some(&monster), player), Reaction::Hostile);
        assert_eq!(reaction(Some(&stormwind_guard), player), Reaction::Friendly);
        assert_eq!(reaction(Some(&orc), player), Reaction::Hostile);
        assert_eq!(reaction(None, player), Reaction::Neutral);
    }

    #[test]
    fn same_faction_under_another_template_is_friendly() {
        let northshire_peasant = row(12, 72, 0, [2, 2, 4], 0, 72);
        let stormwind_guard = row(11, 72, 2081, [3, 2, 12], 0, 0);
        assert_eq!(
            reaction(Some(&stormwind_guard), Some(&northshire_peasant)),
            Reaction::Friendly
        );
    }

    #[test]
    fn hostile_by_default_hates_unrelated_templates() {
        let other = row(2, 2, 0, [0, 0, 0], 0, 0);
        let hater = row(3, 3, HOSTILE_BY_DEFAULT, [0, 0, 0], 0, 0);
        let calm = row(4, 4, 0, [0, 0, 0], 0, 0);
        assert_eq!(reaction(Some(&hater), Some(&other)), Reaction::Hostile);
        assert_eq!(reaction(Some(&calm), Some(&other)), Reaction::Neutral);
    }
}
