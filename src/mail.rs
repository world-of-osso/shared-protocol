//! Mail rules shared by the server and the client's send frame.
//!
//! Ref: AzerothCore `MailHandler.cpp` (postage, delivery delay) and `Mail.cpp`
//! (expiry).

const DAY_SECS: u64 = 86_400;

/// Postage in copper: 30 per attached item, 30 for a letter without items
/// (AzerothCore `MailHandler.cpp:155`, Retail `GetSendMailPrice`).
pub fn postage(attachment_count: usize) -> u64 {
    30 * attachment_count.max(1) as u64
}

/// Mail items sent to a character on another account arrive after an hour
/// (AzerothCore `MailDeliveryDelay`); everything else arrives at once.
pub const ITEM_DELIVERY_DELAY_SECS: u64 = 3_600;

/// Seconds between sending and delivery.
pub fn delivery_delay(has_items: bool, same_account: bool) -> u64 {
    if has_items && !same_account {
        ITEM_DELIVERY_DELAY_SECS
    } else {
        0
    }
}

/// Mail lasts 30 days after delivery (AzerothCore `Mail.cpp:216`).
pub const MAIL_EXPIRY_SECS: u64 = 30 * DAY_SECS;
/// C.O.D. mail lasts 3 days (AzerothCore `Mail.cpp:212`).
pub const COD_MAIL_EXPIRY_SECS: u64 = 3 * DAY_SECS;

/// Seconds between delivery and expiry.
pub fn expiry_secs(cod: u64) -> u64 {
    if cod > 0 {
        COD_MAIL_EXPIRY_SECS
    } else {
        MAIL_EXPIRY_SECS
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn postage_is_thirty_copper_per_item_with_a_letter_minimum() {
        assert_eq!(postage(0), 30);
        assert_eq!(postage(1), 30);
        assert_eq!(postage(12), 360);
    }

    #[test]
    fn only_items_to_another_account_wait_an_hour() {
        assert_eq!(delivery_delay(true, false), 3_600);
        assert_eq!(delivery_delay(true, true), 0);
        assert_eq!(delivery_delay(false, false), 0);
    }

    #[test]
    fn cod_mail_expires_after_three_days_other_mail_after_thirty() {
        assert_eq!(expiry_secs(0), 2_592_000);
        assert_eq!(expiry_secs(1), 259_200);
    }
}
