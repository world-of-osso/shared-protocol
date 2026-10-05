use super::*;
use crate::components::{
    AuraView, CreatureClassification, PowerEntry, PowerType, UnitAuras, UnitFactionTemplate,
    UnitLevel, UnitPowers, UnitRunes, UnitTarget, UnitVignette,
};
use crate::spell_data::CastFailReason;
use bevy_replicon::shared::protocol::ProtocolHasher;
use bevy_replicon::shared::replication::registry::ReplicationRegistry;
use bevy_replicon::shared::replication::rules::ReplicationRules;
use lightyear::prelude::{AppMessageExt, ComponentRegistry};
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::fmt::Debug;

/// Encodes with the same bincode config lightyear uses on the wire, then decodes.
fn assert_wire_round_trip<T: Serialize + DeserializeOwned + PartialEq + Debug>(value: &T) {
    let config = bincode::config::standard();
    let bytes = bincode::serde::encode_to_vec(value, config).unwrap();
    let (decoded, read): (T, usize) = bincode::serde::decode_from_slice(&bytes, config).unwrap();
    assert_eq!(read, bytes.len());
    assert_eq!(&decoded, value);
}

const DB_POWER_TYPES: [(i32, PowerType); 21] = [
    (0, PowerType::Mana),
    (1, PowerType::Rage),
    (2, PowerType::Focus),
    (3, PowerType::Energy),
    (4, PowerType::ComboPoints),
    (5, PowerType::Runes),
    (6, PowerType::RunicPower),
    (7, PowerType::SoulShards),
    (8, PowerType::LunarPower),
    (9, PowerType::HolyPower),
    (10, PowerType::Alternate),
    (11, PowerType::Maelstrom),
    (12, PowerType::Chi),
    (13, PowerType::Insanity),
    (16, PowerType::ArcaneCharges),
    (17, PowerType::Fury),
    (18, PowerType::Pain),
    (19, PowerType::Essence),
    (23, PowerType::AlternateQuest),
    (24, PowerType::AlternateEncounter),
    (25, PowerType::AlternateMount),
];

#[test]
fn power_type_from_db_maps_every_power_type_enum_value() {
    for (id, power) in DB_POWER_TYPES {
        assert_eq!(PowerType::from_db(id), Some(power), "PowerTypeEnum {id}");
        assert_eq!(power as i32, id);
    }
}

#[test]
fn power_type_from_db_rejects_unused_and_out_of_range_ids() {
    for id in [-1, 14, 15, 20, 21, 22, 26, 255, i32::MAX] {
        assert_eq!(PowerType::from_db(id), None, "PowerTypeEnum {id}");
    }
}

#[test]
fn unit_powers_round_trip_keeps_primary_first() {
    let powers = UnitPowers {
        entries: vec![
            PowerEntry {
                power: PowerType::Mana,
                current: 41_250,
                max: 50_000,
                partial: 0,
                regen_per_sec: 0.0,
            },
            PowerEntry {
                power: PowerType::HolyPower,
                current: 3,
                max: 5,
                partial: 0,
                regen_per_sec: 0.0,
            },
            PowerEntry {
                power: PowerType::Essence,
                current: 2,
                max: 5,
                partial: 600,
                regen_per_sec: 0.2,
            },
        ],
        charged_points: vec![2, 5],
    };
    assert_wire_round_trip(&powers);
    assert_wire_round_trip(&UnitRunes {
        duration_ms: 10_000,
        ready_in_ms: vec![0, 0, 0, 4_000, 7_500, 14_000],
    });
    for (_, power) in DB_POWER_TYPES {
        assert_wire_round_trip(&power);
    }
}

#[test]
fn unit_powers_round_trip_preserves_decay_and_fractional_raw_units() {
    assert_wire_round_trip(&UnitPowers {
        entries: vec![PowerEntry {
            power: PowerType::Rage,
            current: 99,
            max: 1000,
            partial: 375,
            regen_per_sec: -12.5,
        }],
        charged_points: Vec::new(),
    });
}

#[test]
fn unit_runes_round_trip_preserves_ready_recharging_and_queued_runes() {
    assert_wire_round_trip(&UnitRunes {
        duration_ms: 8_000,
        ready_in_ms: vec![0, 1_250, 8_000, 9_250, 16_000, 0, 0],
    });
    assert_wire_round_trip(&UnitRunes::default());
}

#[test]
fn unit_auras_round_trip() {
    let auras = UnitAuras {
        auras: vec![
            AuraView {
                instance_id: 1,
                spell_id: 465,
                caster: Some(0x0000_0001_0000_002A),
                stacks: 0,
                charges: 0,
                duration_ms: 0,
                remaining_ms: 0,
                harmful: false,
                dispel_type: 0,
                flags: AuraView::FLAG_PASSIVE | AuraView::FLAG_FROM_PLAYER,
            },
            AuraView {
                instance_id: 7,
                spell_id: 589,
                caster: None,
                stacks: 3,
                charges: 2,
                duration_ms: 16_000,
                remaining_ms: 9_500,
                harmful: true,
                dispel_type: 1,
                flags: AuraView::FLAG_HIDDEN,
            },
        ],
    };
    assert_wire_round_trip(&auras);
}

#[test]
fn unit_level_and_faction_template_round_trip() {
    assert_wire_round_trip(&UnitLevel(70));
    assert_wire_round_trip(&crate::level_scaling::LevelScaling {
        content_tuning_id: 73,
        min_level: 1,
        max_level: 30,
        delta: -4,
    });
    assert_wire_round_trip(&UnitFactionTemplate(1_801));
    assert_wire_round_trip(&UnitTarget(Some(0x0000_0001_0000_002A)));
    assert_wire_round_trip(&UnitTarget(None));
}

/// Timber (world.db creature_template 1132, rank 4) is a rare; Hogger's rank 1 an elite.
#[test]
fn creature_classification_from_world_db_rank_round_trips() {
    let timber = CreatureClassification::from_rank(4).unwrap();
    assert_eq!(timber, CreatureClassification::Rare);
    assert_eq!(timber.token(), "rare");
    assert_wire_round_trip(&timber);
    assert_eq!(
        CreatureClassification::from_rank(2).map(CreatureClassification::token),
        Some("rareelite")
    );
    assert_eq!(
        CreatureClassification::from_rank(3).map(CreatureClassification::token),
        Some("worldboss")
    );
    assert_eq!(CreatureClassification::from_rank(7), None);
}

/// Doomwalker (world.db creature_template 167749) carries Vignette 6520.
#[test]
fn unit_vignette_round_trips() {
    assert_wire_round_trip(&UnitVignette(6_520));
}

#[test]
fn known_spell_messages_round_trip() {
    assert_wire_round_trip(&KnownSpellsSnapshot {
        spells: vec![635, 19_750, 85_673, 275_773],
    });
    assert_wire_round_trip(&SpellsLearned {
        spells: vec![20_271],
    });
    assert_wire_round_trip(&SpellsUnlearned {
        spells: vec![53_600, 85_256],
    });
}

#[test]
fn cooldown_and_charges_round_trip() {
    assert_wire_round_trip(&SpellCooldownUpdate {
        spell_id: 61,
        category: 133,
        duration_ms: 1_500,
        remaining_ms: 1_320,
        is_gcd: true,
    });
    // Surge Forward spending one Skyriding Charge (ChargeCategory 2391, 6, 10.35 s).
    assert_wire_round_trip(&SpellChargesUpdate {
        spell_id: 372_608,
        category: 2_391,
        current: 5,
        max: 6,
        recharge_ms: 10_350,
        remaining_ms: 10_350,
    });
}

#[test]
fn cast_failed_round_trips_every_reason() {
    let reasons = [
        CastFailReason::NoTarget,
        CastFailReason::OutOfRange,
        CastFailReason::InvalidTarget,
        CastFailReason::NotEnoughResource,
        CastFailReason::OnCooldown,
        CastFailReason::OnGlobalCooldown,
        CastFailReason::CasterDead,
        CastFailReason::CantCastWhileMoving,
        CastFailReason::TooClose,
        CastFailReason::SchoolLockedOut,
        CastFailReason::SpellInProgress,
        CastFailReason::NoChargesRemain,
        CastFailReason::NotInFront,
        CastFailReason::NotSupported,
        CastFailReason::Stunned,
        CastFailReason::Silenced,
        CastFailReason::Pacified,
        CastFailReason::Reagents,
        CastFailReason::Totems,
        CastFailReason::InventoryFull,
    ];
    for reason in reasons {
        assert_wire_round_trip(&CastFailed {
            spell_id: 133,
            reason,
            detail: None,
        });
    }
    assert_wire_round_trip(&CastFailed {
        spell_id: 116,
        reason: CastFailReason::OutOfRange,
        detail: Some("Out of range (42 yd)".into()),
    });
}

#[test]
fn combat_log_event_round_trips_every_kind() {
    let misses = [
        MissKind::Miss,
        MissKind::Dodge,
        MissKind::Parry,
        MissKind::Block,
        MissKind::Resist,
        MissKind::Immune,
        MissKind::Evade,
        MissKind::Absorb,
        MissKind::Deflect,
        MissKind::Reflect,
    ];
    let kinds = [
        CombatLogKind::Damage,
        CombatLogKind::Heal,
        CombatLogKind::Energize,
        CombatLogKind::AuraApplied,
        CombatLogKind::AuraRemoved,
        CombatLogKind::AuraRefreshed,
        CombatLogKind::Interrupt,
        CombatLogKind::Dispel,
        CombatLogKind::CastStart,
        CombatLogKind::CastSuccess,
        CombatLogKind::Death,
    ]
    .into_iter()
    .chain(misses.into_iter().map(CombatLogKind::Miss))
    .chain(
        [
            EnvironmentalKind::Fatigue,
            EnvironmentalKind::Drowning,
            EnvironmentalKind::Falling,
        ]
        .map(CombatLogKind::Environmental),
    );
    for kind in kinds {
        assert_wire_round_trip(&CombatLogEvent {
            source: Some(0x0000_0002_0000_0010),
            target: Some(0x0000_0003_0000_0011),
            spell_id: Some(133),
            school_mask: 0b0000_0100,
            amount: 1_234,
            overflow: 56,
            absorbed: 100,
            resisted: 20,
            blocked: 0,
            crit: true,
            glancing: false,
            periodic: false,
            extra_spell_id: Some(116),
            timestamp_unix_ms: 1_791_100_800_123,
            kind,
        });
    }
    assert_wire_round_trip(&CombatLogEvent {
        source: None,
        target: Some(9),
        spell_id: None,
        school_mask: 1,
        amount: -5,
        overflow: 0,
        absorbed: 0,
        resisted: 0,
        blocked: 12,
        crit: false,
        glancing: false,
        periodic: true,
        extra_spell_id: None,
        timestamp_unix_ms: 0,
        kind: CombatLogKind::Miss(MissKind::Block),
    });
}

#[test]
fn action_bar_messages_round_trip() {
    assert_wire_round_trip(&ActionBarSnapshot {
        slots: vec![
            (0, ActionRef::Spell(35_395)),
            (1, ActionRef::Item(6_948)),
            (11, ActionRef::Macro(3)),
        ],
    });
    assert_wire_round_trip(&SetActionButton {
        slot: 4,
        action: Some(ActionRef::Spell(20_271)),
    });
    assert_wire_round_trip(&SetActionButton {
        slot: 119,
        action: None,
    });
}

#[test]
fn trait_and_specialization_messages_round_trip() {
    let entries = vec![
        TraitEntrySelection {
            node_id: 81_600,
            entry_id: 102_600,
            rank: 1,
        },
        TraitEntrySelection {
            node_id: 81_602,
            entry_id: 102_603,
            rank: 2,
        },
    ];
    assert_wire_round_trip(&TraitConfigSnapshot {
        spec_id: 70,
        tree_id: 790,
        entries: entries.clone(),
        unspent: vec![(2_000, 3), (2_001, 0)],
    });
    assert_wire_round_trip(&CommitTraitConfig {
        spec_id: 70,
        entries,
    });
    assert_wire_round_trip(&TraitCommitResult {
        ok: false,
        reason: Some("Not enough points".into()),
    });
    assert_wire_round_trip(&TraitCommitResult {
        ok: true,
        reason: None,
    });
    assert_wire_round_trip(&SpecializationChanged { spec_id: 66 });
    assert_wire_round_trip(&SetSpecialization { spec_id: 65 });
    assert_wire_round_trip(&CancelAura { spell_id: 1_459 });
    assert_wire_round_trip(&CancelMountAura);
}

#[test]
fn protocol_plugin_registers_spell_components_and_messages() {
    let mut app = App::new();
    app.init_resource::<ProtocolHasher>()
        .init_resource::<ReplicationRules>()
        .init_resource::<ReplicationRegistry>();
    app.add_plugins(ProtocolPlugin);

    let components = app.world().resource::<ComponentRegistry>();
    assert!(components.is_registered::<UnitPowers>());
    assert!(components.is_registered::<UnitAuras>());
    assert!(components.is_registered::<UnitLevel>());
    assert!(components.is_registered::<CreatureClassification>());
    assert!(components.is_registered::<UnitVignette>());
    assert!(components.is_registered::<crate::level_scaling::LevelScaling>());
    assert!(components.is_registered::<UnitFactionTemplate>());
    assert!(components.is_registered::<UnitTarget>());

    assert!(app.is_message_registered::<KnownSpellsSnapshot>());
    assert!(app.is_message_registered::<SpellsLearned>());
    assert!(app.is_message_registered::<SpellsUnlearned>());
    assert!(app.is_message_registered::<SpellCooldownUpdate>());
    assert!(app.is_message_registered::<SpellChargesUpdate>());
    assert!(app.is_message_registered::<CastFailed>());
    assert!(app.is_message_registered::<CombatLogEvent>());
    assert!(app.is_message_registered::<SpellGo>());
    assert!(app.is_message_registered::<SpellFailure>());
    assert!(app.is_message_registered::<AttackSwing>());
    assert!(app.is_message_registered::<AttackStop>());
    assert!(app.is_message_registered::<AttackStart>());
    assert!(app.is_message_registered::<AttackStopped>());
    assert!(app.is_message_registered::<ActionBarSnapshot>());
    assert!(app.is_message_registered::<TraitConfigSnapshot>());
    assert!(app.is_message_registered::<TraitCommitResult>());
    assert!(app.is_message_registered::<SpecializationChanged>());
    assert!(app.is_message_registered::<SetActionButton>());
    assert!(app.is_message_registered::<CancelAura>());
    assert!(app.is_message_registered::<CancelMountAura>());
    assert!(app.is_message_registered::<CommitTraitConfig>());
    assert!(app.is_message_registered::<SetSpecialization>());
}

#[test]
fn unit_pose_decodes_trinitycore_values_round_trips_and_replicates() {
    use crate::components::{SheathState, StandState, UnitPose};
    // Stockade Guard 46405 / Petty Criminal 46382 creature_template_addon StandState.
    assert_eq!(StandState::try_from(1), Ok(StandState::Sit));
    assert_eq!(StandState::try_from(3), Ok(StandState::Sleep));
    assert_eq!(StandState::try_from(8), Ok(StandState::Kneel));
    assert_eq!(StandState::try_from(10), Err(10));
    assert_eq!(SheathState::try_from(1), Ok(SheathState::Melee));
    assert_eq!(SheathState::try_from(3), Err(3));
    assert_wire_round_trip(&UnitPose {
        stand_state: StandState::Sleep,
        sheath_state: SheathState::Melee,
        emote_state: 214,
    });

    let mut app = App::new();
    app.init_resource::<ProtocolHasher>()
        .init_resource::<ReplicationRules>()
        .init_resource::<ReplicationRegistry>();
    app.add_plugins(ProtocolPlugin);
    assert!(
        app.world()
            .resource::<ComponentRegistry>()
            .is_registered::<UnitPose>()
    );
}

#[test]
fn player_stand_state_follows_trinitycore_sit_and_stand_predicates_and_replicates() {
    use crate::components::{PlayerStandState, StandState};
    let sitting: Vec<_> = (0..10)
        .map(|value| StandState::try_from(value).unwrap())
        .filter(|state| state.is_sit())
        .collect();
    assert_eq!(
        sitting,
        [
            StandState::Sit,
            StandState::SitChair,
            StandState::SitLowChair,
            StandState::SitMediumChair,
            StandState::SitHighChair
        ]
    );
    let standing: Vec<_> = (0..10)
        .map(|value| StandState::try_from(value).unwrap())
        .filter(|state| state.is_stand())
        .collect();
    assert_eq!(
        standing,
        [StandState::Stand, StandState::Dead, StandState::Submerged]
    );
    assert_wire_round_trip(&PlayerStandState(StandState::Kneel));
    assert_wire_round_trip(&StandStateIntent {
        state: StandState::Sit,
    });

    let mut app = App::new();
    app.init_resource::<ProtocolHasher>()
        .init_resource::<ReplicationRules>()
        .init_resource::<ReplicationRegistry>();
    app.add_plugins(ProtocolPlugin);
    assert!(
        app.world()
            .resource::<ComponentRegistry>()
            .is_registered::<PlayerStandState>()
    );
    assert!(app.is_message_registered::<StandStateIntent>());
}

#[test]
fn spell_go_round_trips_targeted_and_untargeted_casts() {
    assert_wire_round_trip(&SpellGo {
        caster: 0x0000_0002_0000_0010,
        target: Some(0x0000_0003_0000_0011),
        hit_targets: vec![0x0000_0003_0000_0011],
        spell_id: 1464,
    });
    assert_wire_round_trip(&SpellGo {
        caster: 7,
        target: None,
        hit_targets: vec![7, 9],
        spell_id: 6673,
    });
}

#[test]
fn attack_swing_start_and_stop_messages_round_trip() {
    assert_wire_round_trip(&AttackSwing {
        target: 0x0000_0003_0000_0011,
    });
    assert_wire_round_trip(&AttackStop);
    assert_wire_round_trip(&AttackStart {
        attacker: 0x0000_0002_0000_0010,
        victim: 0x0000_0003_0000_0011,
    });
    assert_wire_round_trip(&AttackStopped {
        attacker: 0x0000_0002_0000_0010,
        victim: Some(0x0000_0003_0000_0011),
        now_dead: false,
    });
    assert_wire_round_trip(&AttackStopped {
        attacker: 7,
        victim: None,
        now_dead: true,
    });
}

#[test]
fn spell_failure_round_trips_interrupts_and_failures() {
    assert_wire_round_trip(&SpellFailure {
        caster: 0x0000_0003_0000_0011,
        spell_id: 9053,
        reason: CastFailReason::InterruptedCombat,
        failed_by: Some(0x0000_0002_0000_0010),
    });
    assert_wire_round_trip(&SpellFailure {
        caster: 7,
        spell_id: 133,
        reason: CastFailReason::Interrupted,
        failed_by: None,
    });
}

#[test]
fn spell_cast_intent_round_trips_witness_ray_and_line_of_sight_failure() {
    assert_wire_round_trip(&SpellCastIntent {
        spell_id: Some(133),
        spell: "Fireball".into(),
        target_entity: Some(0x0000_0003_0000_0011),
        witness: Some(WitnessRay {
            start: [-8913.2, 101.5, -553.25],
            end: [-8930.0, 97.1, -560.0],
        }),
    });
    assert_wire_round_trip(&SpellCastIntent {
        spell_id: None,
        spell: "Fireball".into(),
        target_entity: None,
        witness: None,
    });
    assert_wire_round_trip(&SpellFailure {
        caster: 7,
        spell_id: 133,
        reason: CastFailReason::LineOfSight,
        failed_by: None,
    });
}
