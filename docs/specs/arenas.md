# Arena wire contract

QueueForArena explicitly distinguishes Skirmish2v2, Skirmish3v3 and SoloShuffle; server validates source policy/eligibility. Queue snapshots preserve mode. Existing match-owned battleground invitations and leave messages are reused for arena copies.

BattlegroundSnapshot.arena is absent for battlegrounds. Arena payload contains mode, optional source preparation countdown, Shadow Sight availability and authoritative participant name/class/team/current/maxhealth. No client-generated outcome, rating or currency authority.

BgObjective.Elimination represents arena end conditions. Full protocol tests are required after arena protocol changes.
