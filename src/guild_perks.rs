//! Mainline GuildPerkSpells definitions joined to installed spell names/icons.
use serde::Deserialize;
use std::{collections::HashMap, io::Read, path::Path};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GuildPerk {
    pub id: u32,
    pub spell_id: u32,
    pub name: String,
    pub icon_fdid: u32,
}

#[derive(Deserialize)]
struct PerkRow {
    #[serde(rename = "ID")]
    id: u32,
    #[serde(rename = "SpellID")]
    spell_id: u32,
}
#[derive(Deserialize)]
struct NameRow {
    #[serde(rename = "ID")]
    id: u32,
    #[serde(rename = "Name_lang")]
    name: String,
}
#[derive(Deserialize)]
struct MiscRow {
    #[serde(rename = "SpellID")]
    spell_id: u32,
    #[serde(rename = "SpellIconFileDataID")]
    icon: u32,
}

pub fn load_guild_perks(root: &Path) -> Result<Vec<GuildPerk>, String> {
    let open = |name: &str| {
        let path = root.join(format!("{name}.csv"));
        std::fs::File::open(&path).map_err(|error| format!("Read {}: {error}", path.display()))
    };
    read_guild_perks(
        open("GuildPerkSpells")?,
        open("SpellName")?,
        open("SpellMisc")?,
    )
}

fn read_guild_perks(
    perks: impl Read,
    names: impl Read,
    misc: impl Read,
) -> Result<Vec<GuildPerk>, String> {
    let mut rows = csv::Reader::from_reader(perks)
        .deserialize::<PerkRow>()
        .map(|row| {
            row.map(|row| GuildPerk {
                id: row.id,
                spell_id: row.spell_id,
                name: String::new(),
                icon_fdid: 0,
            })
            .map_err(|error| format!("GuildPerkSpells: {error}"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    rows.sort_by_key(|row| row.id);
    let positions: HashMap<_, _> = rows
        .iter()
        .enumerate()
        .map(|(index, row)| (row.spell_id, index))
        .collect();
    read_perk_names(&mut rows, &positions, names)?;
    read_perk_icons(&mut rows, &positions, misc)?;
    for row in &rows {
        if row.name.is_empty() || row.icon_fdid == 0 {
            return Err(format!(
                "GuildPerkSpells row {} requires installed metadata for spell {}",
                row.id, row.spell_id
            ));
        }
    }
    Ok(rows)
}

fn read_perk_names(
    rows: &mut [GuildPerk],
    positions: &HashMap<u32, usize>,
    input: impl Read,
) -> Result<(), String> {
    for row in csv::Reader::from_reader(input).deserialize::<NameRow>() {
        let row = row.map_err(|error| format!("SpellName: {error}"))?;
        if let Some(&index) = positions.get(&row.id) {
            rows[index].name = row.name;
        }
    }
    Ok(())
}

fn read_perk_icons(
    rows: &mut [GuildPerk],
    positions: &HashMap<u32, usize>,
    input: impl Read,
) -> Result<(), String> {
    for row in csv::Reader::from_reader(input).deserialize::<MiscRow>() {
        let row = row.map_err(|error| format!("SpellMisc: {error}"))?;
        if let Some(&index) = positions.get(&row.spell_id) {
            rows[index].icon_fdid = row.icon;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    const PERKS: &str = "ID,SpellID\n16,83951\n2,78633\n14,83950\n7,83944\n10,83958\n";
    const NAMES: &str = "ID,Name_lang\n78633,Mount Up\n83944,Hasty Hearth\n83958,Mobile Banking\n83950,The Quick and the Dead\n83951,Guild Mail\n";
    const MISC: &str = "SpellID,SpellIconFileDataID\n78633,413588\n83944,413582\n83958,413587\n83950,413591\n83951,413580\n";

    #[test]
    fn guild_perks_reads_real_definitions_in_db2_order_with_actual_metadata() {
        let rows = read_guild_perks(PERKS.as_bytes(), NAMES.as_bytes(), MISC.as_bytes()).unwrap();
        assert_eq!(
            rows.iter()
                .map(|row| (row.id, row.spell_id, row.name.as_str(), row.icon_fdid))
                .collect::<Vec<_>>(),
            vec![
                (2, 78633, "Mount Up", 413588),
                (7, 83944, "Hasty Hearth", 413582),
                (10, 83958, "Mobile Banking", 413587),
                (14, 83950, "The Quick and the Dead", 413591),
                (16, 83951, "Guild Mail", 413580),
            ]
        );
    }

    #[test]
    fn guild_perks_refuses_missing_metadata_instead_of_fabricating_a_name_or_icon() {
        let incomplete = MISC.replace("83958,413587\n", "");
        let error = read_guild_perks(PERKS.as_bytes(), NAMES.as_bytes(), incomplete.as_bytes())
            .unwrap_err();
        assert!(error.contains("83958"), "{error}");
    }
}
