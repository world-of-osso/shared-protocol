//! Authenticated Retail70391 PVPDifficulty; see docs/specs/battleground-brackets.md.
use std::sync::OnceLock;

use serde::Deserialize;

use super::LevelBracket;

#[derive(Deserialize)]
struct DifficultyRow {
    #[serde(rename = "MapID")]
    map_id: u32,
    #[serde(rename = "MinLevel")]
    min_level: u8,
    #[serde(rename = "MaxLevel")]
    max_level: u8,
}

fn source_rows() -> &'static [DifficultyRow] {
    static ROWS: OnceLock<Vec<DifficultyRow>> = OnceLock::new();
    ROWS.get_or_init(|| {
        let source = include_bytes!("../scripts/reference/pvp-12.1.5.70391/PVPDifficulty.csv");
        csv::Reader::from_reader(source.as_slice())
            .deserialize()
            .collect::<Result<Vec<_>, _>>()
            .expect("bundled PVPDifficulty source must decode")
    })
}

/// Exact source range; neither template admission bounds nor decade arithmetic.
pub fn source_level_bracket(map_id: u32, level: u8) -> Option<LevelBracket> {
    source_rows()
        .iter()
        .find(|row| row.map_id == map_id && (row.min_level..=row.max_level).contains(&level))
        .map(|row| LevelBracket::new(row.min_level, row.max_level))
}
