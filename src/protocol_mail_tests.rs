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

fn letter(from_player: bool, returnable: bool) -> MailHeader {
    MailHeader {
        mail_id: 4,
        sender: "Tradea".into(),
        subject: "Linen".into(),
        body: "For your tailoring.".into(),
        money: 0,
        cod: 0,
        attachments: Vec::new(),
        expires_at: 1_800_000_000,
        read: false,
        returned: from_player && !returnable,
        from_player,
        returnable,
    }
}

fn linen_attachment() -> MailAttachment {
    MailAttachment {
        slot: 2,
        item: ItemStack {
            item_guid: 90,
            item_id: 2589,
            definition_source: crate::item_data::ItemDefinitionSource::Retail,
            count: 20,
            durability: None,
            soulbound: false,
        },
        name: "Linen Cloth".into(),
        quality: 1,
    }
}

#[test]
fn player_mail_holding_items_or_money_is_returned_not_deleted() {
    assert!(letter(true, true).can_delete());
    let mut with_item = letter(true, true);
    with_item.attachments.push(linen_attachment());
    assert!(!with_item.can_delete());
    let mut with_money = letter(true, true);
    with_money.money = 1;
    assert!(!with_money.can_delete());
    // Returned mail and mail from the Auction House cannot go back.
    let mut returned = letter(true, false);
    returned.money = 1;
    assert!(returned.can_delete());
    let mut auction = letter(false, false);
    auction.attachments.push(linen_attachment());
    assert!(auction.can_delete());
}

#[test]
fn mail_messages_round_trip() {
    let mut header = letter(true, true);
    header.cod = 5_000_000_000;
    header.attachments.push(linen_attachment());
    assert_wire_round_trip(&MailboxContents {
        object: 9,
        mails: vec![header],
        now: 1_790_000_000,
    });
    assert_wire_round_trip(&SendMail {
        object: 9,
        recipient: "Tradeb".into(),
        subject: "Linen".into(),
        body: String::new(),
        attachments: vec![90, 91],
        money: 10_000_000_000,
        cod: 0,
    });
    assert_wire_round_trip(&MailRequest {
        object: 9,
        mail_id: 4,
        action: MailAction::TakeAttachment { slot: 2 },
    });
    assert_wire_round_trip(&MailSent { object: 9 });
    assert_wire_round_trip(&MailFailed {
        object: 9,
        error: MailError::CodInsufficientMoney,
    });
    assert_wire_round_trip(&PendingMail {
        senders: vec!["Tradea".into(), "Auction House".into()],
    });
    assert_wire_round_trip(&InteractionOpened {
        npc: 9,
        kind: InteractionKind::Role(NpcRole::Mailbox),
    });
}

#[test]
fn mail_errors_use_retail_wording() {
    assert_eq!(
        MailError::ToSelf.message(),
        "You can't send mail to yourself."
    );
    assert_eq!(
        MailError::RecipientNotFound.message(),
        "Cannot find mail recipient."
    );
    assert_eq!(
        MailError::BoundItem.message(),
        "You can't mail soulbound items."
    );
}

#[test]
fn protocol_plugin_registers_mail_messages() {
    let mut app = App::new();
    app.init_resource::<ProtocolHasher>()
        .init_resource::<ReplicationRules>()
        .init_resource::<ReplicationRegistry>();
    app.add_plugins(ProtocolPlugin);

    assert!(app.is_message_registered::<MailboxContents>());
    assert!(app.is_message_registered::<SendMail>());
    assert!(app.is_message_registered::<MailRequest>());
    assert!(app.is_message_registered::<MailSent>());
    assert!(app.is_message_registered::<MailFailed>());
    assert!(app.is_message_registered::<PendingMail>());
    assert!(app.is_message_registered::<CancelTradeAccept>());
}
