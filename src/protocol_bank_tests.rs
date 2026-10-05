use super::*;
use bevy_replicon::shared::protocol::ProtocolHasher;
use bevy_replicon::shared::replication::registry::ReplicationRegistry;
use bevy_replicon::shared::replication::rules::ReplicationRules;
use lightyear::prelude::AppMessageExt;
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::fmt::Debug;

fn assert_wire_round_trip<T: Serialize + DeserializeOwned + PartialEq + Debug>(value: &T) {
    let config = bincode::config::standard();
    let bytes = bincode::serde::encode_to_vec(value, config).unwrap();
    let (decoded, read): (T, usize) = bincode::serde::decode_from_slice(&bytes, config).unwrap();
    assert_eq!(read, bytes.len());
    assert_eq!(&decoded, value);
}

fn linen(guid: u64, count: u32) -> ItemStack {
    ItemStack {
        item_guid: guid,
        item_id: 2589,
        definition_source: crate::item_data::ItemDefinitionSource::Retail,
        count,
        durability: None,
        soulbound: false,
    }
}

#[test]
fn bank_messages_round_trip() {
    let mut slots = vec![None; BANK_TAB_SLOTS];
    slots[3] = Some(linen(90, 20));
    assert_wire_round_trip(&BankContents {
        bank: BankType::Account,
        tabs: vec![BankTabView {
            name: "Warband Tab 1".into(),
            icon: 134400,
            deposit_flags: bank_tab_flags::REAGENTS | bank_tab_flags::JUNK,
            slots,
        }],
        next_tab_cost: Some(2_500_000_000),
        money: Some(12_345),
    });
    assert_wire_round_trip(&BankDeposit {
        npc: 7,
        bank: BankType::Character,
        tab: 1,
        item_guid: 90,
    });
    assert_wire_round_trip(&BankWithdraw {
        npc: 7,
        bank: BankType::Account,
        tab: 0,
        slot: 97,
    });
    assert_wire_round_trip(&BankPurchaseTab {
        npc: 7,
        bank: BankType::Character,
    });
    assert_wire_round_trip(&BankMoneyTransfer {
        npc: 7,
        bank: BankType::Account,
        copper: 10_000,
        deposit: true,
    });
    assert_wire_round_trip(&BankAutoDeposit {
        npc: 7,
        bank: BankType::Account,
        include_reagents: true,
    });
    assert_wire_round_trip(&BankUpdateTabSettings {
        npc: 7,
        bank: BankType::Character,
        tab: 0,
        name: "Cloth".into(),
        icon: 132889,
        deposit_flags: bank_tab_flags::PROFESSION_GOODS,
    });
    assert_wire_round_trip(&BankFailed {
        npc: 7,
        error: BankError::SoulboundInAccountBank,
    });
}

#[test]
fn guild_bank_messages_round_trip() {
    let mut slots = vec![None; GUILD_BANK_TAB_SLOTS];
    slots[0] = Some(linen(91, 5));
    assert_wire_round_trip(&GuildBankContents {
        object: 9,
        guild_name: "Bank Testers".into(),
        tabs: vec![GuildBankTabView {
            name: "Tab 1".into(),
            icon: 0,
            viewable: true,
            can_deposit: true,
            withdrawals_per_day: Some(2),
            remaining_withdrawals: Some(1),
            text: "Linen for the tailors".into(),
            slots,
        }],
        money: 1_000_000,
        withdraw_money_remaining: None,
        next_tab_cost: Some(2_500_000),
        is_leader: true,
    });
    assert_wire_round_trip(&GuildBankDeposit {
        object: 9,
        tab: 0,
        item_guid: 91,
    });
    assert_wire_round_trip(&GuildBankWithdraw {
        object: 9,
        tab: 0,
        slot: 0,
    });
    assert_wire_round_trip(&GuildBankMoneyTransfer {
        object: 9,
        copper: 500,
        deposit: false,
    });
    assert_wire_round_trip(&GuildBankBuyTab { object: 9 });
    assert_wire_round_trip(&GuildBankSetTabInfo {
        object: 9,
        tab: 0,
        name: "Cloth".into(),
        icon: 132889,
    });
    assert_wire_round_trip(&GuildBankSetTabText {
        object: 9,
        tab: 0,
        text: "Take what you need".into(),
    });
    assert_wire_round_trip(&GuildBankQueryLog {
        object: 9,
        tab: None,
    });
    assert_wire_round_trip(&GuildBankLog {
        tab: Some(0),
        entries: vec![GuildBankLogEntry {
            kind: GuildBankLogKind::WithdrawItem,
            actor: "Bankalt".into(),
            item_id: 2589,
            item_name: "Linen Cloth".into(),
            count: 5,
            copper: 0,
            seconds_ago: 90,
        }],
    });
    // GUILDBANK_REPAIR_MONEY_FORMAT "%s withdrew %s for repairs".
    assert_wire_round_trip(&GuildBankLog {
        tab: None,
        entries: vec![GuildBankLogEntry {
            kind: GuildBankLogKind::RepairMoney,
            actor: "Bankalt".into(),
            item_id: 0,
            item_name: String::new(),
            count: 0,
            copper: 52,
            seconds_ago: 5,
        }],
    });
    assert_wire_round_trip(&GuildBankFailed {
        object: 9,
        error: GuildBankError::WithdrawLimit,
    });
}

#[test]
fn game_object_use_round_trips() {
    assert_wire_round_trip(&UseGameObject { object: 9 });
    assert_wire_round_trip(&GameObjectInfo {
        entry: 187329,
        go_type: GAMEOBJECT_TYPE_GUILD_BANK,
        display_id: 7607,
        name: "Guild Vault".into(),
        scale: 1.0,
    });
    assert_wire_round_trip(&InteractionOpened {
        npc: 9,
        kind: InteractionKind::Role(NpcRole::GuildBanker),
    });
}

#[test]
fn vaults_mailboxes_stones_portals_and_chairs_are_usable_and_stockade_fires_are_not() {
    let object = |go_type, name: &str| GameObjectInfo {
        entry: 0,
        go_type,
        display_id: 0,
        name: name.into(),
        scale: 1.0,
    };
    for usable in [
        object(GAMEOBJECT_TYPE_GUILD_BANK, "Guild Vault"),
        object(GAMEOBJECT_TYPE_MAILBOX, "Mailbox"),
        object(GAMEOBJECT_TYPE_MEETINGSTONE, "Meeting Stone"),
        object(GAMEOBJECT_TYPE_RITUAL, "Summoning Portal"),
        // Lion's Pride Inn 177499.
        object(GAMEOBJECT_TYPE_CHAIR, "Wooden Chair"),
    ] {
        assert!(usable.is_usable(), "{}", usable.name);
    }
    // Stockade 206038 Small Fire (0.5) and 206117 Bonfire.
    assert!(!object(GAMEOBJECT_TYPE_GENERIC, "Small Fire (0.5)").is_usable());
    assert!(!object(GAMEOBJECT_TYPE_SPELL_FOCUS, "Bonfire").is_usable());
}

#[test]
fn bank_errors_use_retail_wording() {
    assert_eq!(BankError::BankFull.message(), "Your bank is full");
    assert_eq!(
        BankError::SoulboundInAccountBank.message(),
        "Soulbound items cannot be stored in the Warband Bank."
    );
    assert_eq!(
        GuildBankError::BoundItem.message(),
        "You cannot store soulbound items in the guild bank"
    );
    assert_eq!(
        GuildBankError::WithdrawLimit.message(),
        "You cannot withdraw that much from the guild bank."
    );
}

#[test]
fn protocol_plugin_registers_bank_messages() {
    let mut app = App::new();
    app.init_resource::<ProtocolHasher>()
        .init_resource::<ReplicationRules>()
        .init_resource::<ReplicationRegistry>();
    app.add_plugins(ProtocolPlugin);

    assert!(app.is_message_registered::<BankContents>());
    assert!(app.is_message_registered::<BankFailed>());
    assert!(app.is_message_registered::<BankDeposit>());
    assert!(app.is_message_registered::<BankWithdraw>());
    assert!(app.is_message_registered::<BankPurchaseTab>());
    assert!(app.is_message_registered::<BankMoneyTransfer>());
    assert!(app.is_message_registered::<BankAutoDeposit>());
    assert!(app.is_message_registered::<BankUpdateTabSettings>());
    assert!(app.is_message_registered::<GuildBankContents>());
    assert!(app.is_message_registered::<GuildBankLog>());
    assert!(app.is_message_registered::<GuildBankFailed>());
    assert!(app.is_message_registered::<GuildBankDeposit>());
    assert!(app.is_message_registered::<GuildBankWithdraw>());
    assert!(app.is_message_registered::<GuildBankMoneyTransfer>());
    assert!(app.is_message_registered::<GuildBankBuyTab>());
    assert!(app.is_message_registered::<GuildBankSetTabInfo>());
    assert!(app.is_message_registered::<GuildBankSetTabText>());
    assert!(app.is_message_registered::<GuildBankQueryLog>());
    assert!(app.is_message_registered::<UseGameObject>());
}
