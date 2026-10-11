# Quest sharing and timers

Ordered QuestChannel adds PushQuestToParty, server-issued QuestShareOffer, QuestPushResponse (offer ID + accept/decline), and QuestPushResult. An offer is addressed to one character; server revalidates party membership, source quest and recipient eligibility on response. Results name Retail QuestPushReason outcomes; clients render local GlobalStrings ERR_QUEST_PUSH_* text.

QuestEntrySnapshot carries expires_at (absolute server Unix seconds) and failed. Server owns expiry and reward rejection. Timed sharing inherits remaining source deadline. Daily/weekly eligibility stays server-side, using persisted per-character reward timestamps and the realm's local schedule.

Wire schema changes require deployment gate protocol → server → engine. No mixed-version compatibility.
