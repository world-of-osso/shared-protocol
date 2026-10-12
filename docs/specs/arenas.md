# Arena wire contract

QueueForArena explicitly distinguishes Skirmish2v2, Skirmish3v3, SoloShuffle and rated2v2/3v3 modes; rated solo-message requests are rejected in favor of full-group QueueForRatedPvp. Arena results carry per-player pre/post personal rating and MMR. GameObjectAnimation carries source M2 sequences; GameObjectVisible hides consumed orbs without invalidating aura caster ownership; server validates source policy/eligibility. Queue snapshots preserve mode. Existing match-owned battleground invitations and leave messages are reused for arena copies.

BattlegroundSnapshot.arena is absent for battlegrounds. Arena payload contains mode, optional source preparation countdown, Shadow Sight availability and authoritative participant entity/name/class/team/current/maxhealth. No client-generated outcome, rating or currency authority.

BgObjective.Elimination represents arena end conditions. Full protocol tests are required after arena protocol changes.
