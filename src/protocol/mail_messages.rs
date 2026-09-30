//! Mail messages (Retail `Blizzard_MailFrame`, AzerothCore `MailHandler.cpp`).
//!
//! The mailbox opens at a Mailbox game object (`UseGameObject`, answered with
//! `InteractionOpened` / `NpcRole::Mailbox`). The server then sends
//! `MailboxContents`, and again after every change while the mailbox stays open.
//! Requests name the mailbox by server entity bits and are refused unless its mail
//! frame is open; refusals come back as `MailFailed`. `PendingMail` (Retail
//! `UPDATE_PENDING_MAIL`) is sent on entering the world and whenever the unread
//! delivered mail changes, open mailbox or not. All messages use `MailChannel`.

use crate::protocol::ProtocolRegistrationExt;
use bevy::prelude::*;
use lightyear::prelude::{AppChannelExt, ChannelMode, ChannelSettings, NetworkDirection};
use serde::{Deserialize, Serialize};

use super::ItemStack;

/// Reliable ordered channel for mail messages, bidirectional.
pub struct MailChannel;

/// AzerothCore `MAX_MAIL_ITEMS`, Retail `ATTACHMENTS_MAX_SEND`.
pub const MAX_MAIL_ATTACHMENTS: usize = 12;
/// Retail `SendMailSubjectEditBox` letters.
pub const MAIL_SUBJECT_MAX_LETTERS: usize = 64;
/// Retail `SendMailBodyEditBox` letters.
pub const MAIL_BODY_MAX_LETTERS: usize = 500;

/// One attached item; `slot` is its fixed attachment index (Retail
/// `TakeInboxItem(index, attachmentIndex)`), kept when other attachments are taken.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct MailAttachment {
    pub slot: u8,
    pub item: ItemStack,
    /// Item name and Retail quality, for the attachment tooltip and the
    /// `DELETE_MAIL_CONFIRMATION` text.
    pub name: String,
    pub quality: u8,
}

/// One inbox entry (Retail `GetInboxHeaderInfo` + `GetInboxText`).
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct MailHeader {
    pub mail_id: u64,
    pub sender: String,
    pub subject: String,
    pub body: String,
    /// Copper attached.
    pub money: u64,
    /// Copper the recipient pays to take the attachments.
    pub cod: u64,
    pub attachments: Vec<MailAttachment>,
    /// Unix seconds when the mail is returned to its sender or deleted.
    pub expires_at: u64,
    pub read: bool,
    /// Sent back to its sender (`wasReturned`).
    pub returned: bool,
    /// Sent by a player (`canReply`).
    pub from_player: bool,
    /// Player mail not yet returned and not a C.O.D. payment can go back to its
    /// sender (`ReturnInboxItem`).
    pub returnable: bool,
}

impl MailHeader {
    /// Retail `InboxItemCanDelete`: returnable mail still holding items or money is
    /// returned rather than deleted.
    pub fn can_delete(&self) -> bool {
        !self.returnable || (self.attachments.is_empty() && self.money == 0)
    }
}

/// The open mailbox's inbox, newest mail first.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct MailboxContents {
    pub object: u64,
    pub mails: Vec<MailHeader>,
    /// Server unix seconds when sent, for the days-left column.
    pub now: u64,
}

/// Retail `SendMail(recipient, subject, body)` with the attachments and the
/// `SetSendMailMoney` / `SetSendMailCOD` amount (at most one of `money`, `cod`).
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct SendMail {
    pub object: u64,
    pub recipient: String,
    pub subject: String,
    pub body: String,
    /// Bag item guids, at most `MAX_MAIL_ATTACHMENTS`.
    pub attachments: Vec<u64>,
    pub money: u64,
    pub cod: u64,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum MailAction {
    /// Opening a mail (`GetInboxText`).
    MarkRead,
    /// `TakeInboxItem`; pays the COD first when there is one.
    TakeAttachment { slot: u8 },
    /// `TakeInboxMoney`.
    TakeMoney,
    /// `ReturnInboxItem`.
    Return,
    /// `DeleteInboxItem`.
    Delete,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct MailRequest {
    pub object: u64,
    pub mail_id: u64,
    pub action: MailAction,
}

/// The mail was sent (Retail `MAIL_SEND_SUCCESS`).
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct MailSent {
    pub object: u64,
}

/// Why the server refused a mail request; `message` is the Retail UI text.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum MailError {
    NotInteracting,
    RecipientNotFound,
    ToSelf,
    WrongFaction,
    RecipientMailboxFull,
    NotEnoughMoney,
    TooManyAttachments,
    InvalidAttachment,
    BoundItem,
    CodInsufficientMoney,
    InventoryFull,
    TooMuchGold,
    CannotDelete,
    Internal,
}

impl MailError {
    pub fn message(self) -> &'static str {
        match self {
            Self::NotInteracting => "You are too far away.", // ERR_USE_TOO_FAR
            Self::RecipientNotFound => "Cannot find mail recipient.", // ERR_MAIL_TARGET_NOT_FOUND
            Self::ToSelf => "You can't send mail to yourself.", // ERR_MAIL_TO_SELF
            Self::WrongFaction => "Target is unfriendly.",   // ERR_PLAYER_WRONG_FACTION
            Self::RecipientMailboxFull => "Recipient can't receive mail at this time.", // ERR_MAIL_RECEPIENT_CANT_RECEIVE_MAIL
            Self::NotEnoughMoney => "You don't have enough money.", // ERR_NOT_ENOUGH_MONEY
            Self::TooManyAttachments => "A mail included too many attachments.", // ERR_MAIL_TOO_MANY_ATTACHMENTS
            Self::InvalidAttachment => "A mail attachment was invalid.", // ERR_MAIL_INVALID_ATTACHMENT
            Self::BoundItem => "You can't mail soulbound items.",        // ERR_MAIL_BOUND_ITEM
            Self::CodInsufficientMoney => {
                "You do not have enough money to pay the C.O.D. charges." // COD_INSUFFICIENT_MONEY
            }
            Self::InventoryFull => "Inventory is full.", // ERR_INV_FULL
            Self::TooMuchGold => "At gold limit",        // ERR_TOO_MUCH_GOLD
            Self::CannotDelete => "You can't delete that.", // ERR_MAIL_DELETE_ITEM_ERROR
            Self::Internal => "Internal mail database error.", // ERR_MAIL_DATABASE_ERROR
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct MailFailed {
    pub object: u64,
    pub error: MailError,
}

/// Unread delivered mail (Retail `HasNewMail` / `GetLatestThreeSenders`): the
/// senders of the newest three, empty when there is none.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct PendingMail {
    pub senders: Vec<String>,
}

pub(super) fn register_mail_protocol(app: &mut App) {
    app.add_channel::<MailChannel>(ChannelSettings {
        mode: ChannelMode::OrderedReliable(default()),
        ..default()
    })
    .add_direction(NetworkDirection::Bidirectional);
    app.register_protocol_message::<MailboxContents>()
        .add_direction(NetworkDirection::ServerToClient);
    app.register_protocol_message::<MailSent>()
        .add_direction(NetworkDirection::ServerToClient);
    app.register_protocol_message::<MailFailed>()
        .add_direction(NetworkDirection::ServerToClient);
    app.register_protocol_message::<PendingMail>()
        .add_direction(NetworkDirection::ServerToClient);
    app.register_protocol_message::<SendMail>()
        .add_direction(NetworkDirection::ClientToServer);
    app.register_protocol_message::<MailRequest>()
        .add_direction(NetworkDirection::ClientToServer);
}
