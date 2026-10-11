# Retail guild charters

`GuildCharterRequest` and `GuildCharterReply` use the existing reliable GuildChannel.
Clients reference owned charter item GUIDs; NPC and account authority stays server-side.

## Contract

- Registrar, purchase, owned-item show, offer to a named online character, sign,
  rename and turn-in have distinct requests and authoritative replies.
- Registrar data contains the authentic NPC name, charter price and optional owned
  charter GUID. A purchased charter closes the registrar; using the item opens PetitionFrame.
- Petition data contains the proposed guild name, owner and actual signer names,
  required signature count and actor-specific owner/signing permission.
- The server persists petitions, unique signer accounts and charter ownership;
  creation consumes the owned item and commits membership/ranks together.
- Both skins share the same Mainline controls and mechanics.

## References

- Retail `Blizzard_UIPanels_Game/Mainline/{GuildRegistrarFrame,PetitionFrame}.{lua,xml}`.
- TrinityCore `a352b1fa9017138cc2f1dcfc0a51afaa17391ecf`,
  `src/server/game/Handlers/PetitionsHandler.cpp`: petitioner flag, item5863,
  purchase/rename/offer/sign/turn-in and account deduplication.
- Pinned `worldserver.conf.dist`: Guild.CharterCost1000, MinPetitionSigns4.

`PetitionManager` remains the pure domain model; IDs inside it are not network
credentials. Pending titles do not reserve guild names until creation succeeds.
