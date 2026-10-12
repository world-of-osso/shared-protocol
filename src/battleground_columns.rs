//! Scoreboard objective columns: PvpStat.db2 (FDID2992919) Description per stat and
//! PvpScoreboardLayout.db2 (FDID2992918) OrderIndex, local build12.1.0.69933.
//! Blizzard_PVPMatch/PVPMatchTable.lua sorts GetMatchPVPStatColumns by orderIndex.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PvpStatColumn {
    pub stat: u32,
    pub title: &'static str,
}

const fn column(stat: u32, title: &'static str) -> PvpStatColumn {
    PvpStatColumn { stat, title }
}

// Order0 Flag Returns, order1 Flag Captures.
const WARSONG: [PvpStatColumn; 2] = [column(929, "Flag Returns"), column(928, "Flag Captures")];
const ARATHI: [PvpStatColumn; 2] = [
    column(926, "Bases Assaulted"),
    column(927, "Bases Defended"),
];
const GILNEAS: [PvpStatColumn; 2] = [
    column(370, "Bases Assaulted"),
    column(371, "Bases Defended"),
];
const TWIN_PEAKS: [PvpStatColumn; 2] = [column(290, "Flag Captures"), column(291, "Flag Returns")];

/// Columns in source order for one battleground map; empty when the map has none.
pub fn pvp_stat_columns(map_id: u32) -> &'static [PvpStatColumn] {
    match map_id {
        2106 => &WARSONG,
        2107 => &ARATHI,
        761 => &GILNEAS,
        726 => &TWIN_PEAKS,
        _ => &[],
    }
}
