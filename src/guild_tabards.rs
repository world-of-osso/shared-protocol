//! Guild tabard DB2 records shared by server validation and client appearance.
//! csv is used because these authored CSV tables must not have a second ad-hoc parser.
use crate::guild_creation::GuildTabardDesign;
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

#[derive(Debug, Clone, Default)]
pub struct GuildTabardCatalog {
    pub background_colors: BTreeMap<u8, [u8; 3]>,
    pub border_colors: BTreeMap<u8, [u8; 3]>,
    pub emblem_colors: BTreeMap<u8, [u8; 3]>,
    pub emblems: BTreeMap<u8, u32>,
    backgrounds: BTreeMap<(u8, u8, u8), u32>,
    borders: BTreeMap<(u8, u8, u8, u8), u32>,
    symbols: BTreeMap<(u8, u8, u8), u32>,
}

/// Character components3/4: torso upper/lower, ordered background→emblem→border.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GuildTabardTextures {
    pub upper: [u32; 3],
    pub lower: [u32; 3],
}

impl GuildTabardCatalog {
    pub fn load(root: &Path) -> Result<Self, String> {
        let mut catalog = Self {
            background_colors: load_colors(root, "GuildColorBackground")?,
            border_colors: load_colors(root, "GuildColorBorder")?,
            emblem_colors: load_colors(root, "GuildColorEmblem")?,
            ..Self::default()
        };
        catalog.load_textures(root)?;
        Ok(catalog)
    }

    fn load_textures(&mut self, root: &Path) -> Result<(), String> {
        for row in load_rows(root, "GuildEmblem", &["EmblemID", "TextureFileDataID"])? {
            self.emblems.insert(byte(row[0])?, row[1]);
        }
        for row in load_rows(
            root,
            "GuildTabardBackground",
            &["Tier", "Component", "Color", "FileDataID"],
        )? {
            self.backgrounds
                .insert((byte(row[0])?, byte(row[1])?, byte(row[2])?), row[3]);
        }
        for row in load_rows(
            root,
            "GuildTabardBorder",
            &["BorderID", "Tier", "Component", "Color", "FileDataID"],
        )? {
            self.borders.insert(
                (byte(row[0])?, byte(row[1])?, byte(row[2])?, byte(row[3])?),
                row[4],
            );
        }
        for row in load_rows(
            root,
            "GuildTabardEmblem",
            &["EmblemID", "Component", "Color", "FileDataID"],
        )? {
            self.symbols
                .insert((byte(row[0])?, byte(row[1])?, byte(row[2])?), row[3]);
        }
        Ok(())
    }

    pub fn validate(&self, design: GuildTabardDesign) -> Result<(), String> {
        // TC EmblemInfo::ValidateEmblemColors looks up these three DB2 stores.
        if !self
            .background_colors
            .contains_key(&design.background_color)
            || !self.border_colors.contains_key(&design.border_color)
            || !self.emblem_colors.contains_key(&design.icon_color)
        {
            return Err("Invalid guild emblem colors.".into());
        }
        if !self.emblems.contains_key(&design.icon) {
            return Err("Invalid guild emblem style.".into());
        }
        self.textures(design, 0).map(|_| ())
    }

    pub fn border_styles(&self) -> Vec<u8> {
        self.borders
            .keys()
            .map(|key| key.0)
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect()
    }

    pub fn textures(
        &self,
        design: GuildTabardDesign,
        tier: u8,
    ) -> Result<GuildTabardTextures, String> {
        Ok(GuildTabardTextures {
            upper: self.component_textures(design, tier, 3)?,
            lower: self.component_textures(design, tier, 4)?,
        })
    }

    fn component_textures(
        &self,
        design: GuildTabardDesign,
        tier: u8,
        component: u8,
    ) -> Result<[u32; 3], String> {
        let background = self
            .backgrounds
            .get(&(tier, component, design.background_color));
        let emblem = self
            .symbols
            .get(&(design.icon, component, design.icon_color));
        let border = self
            .borders
            .get(&(design.border, tier, component, design.border_color));
        match (background, emblem, border) {
            (Some(background), Some(emblem), Some(border)) => Ok([*background, *emblem, *border]),
            _ => Err(format!(
                "Guild tabard{design:?} tier{tier} component{component} has no authored textures"
            )),
        }
    }
}

fn byte(value: u32) -> Result<u8, String> {
    u8::try_from(value).map_err(|_| format!("Guild tabard catalog ID{value} exceeds byte range"))
}

fn load_colors(root: &Path, table: &str) -> Result<BTreeMap<u8, [u8; 3]>, String> {
    load_rows(root, table, &["ID", "Red", "Green", "Blue"])?
        .into_iter()
        .map(|row| Ok((byte(row[0])?, [byte(row[1])?, byte(row[2])?, byte(row[3])?])))
        .collect()
}

fn load_rows(root: &Path, table: &str, fields: &[&str]) -> Result<Vec<Vec<u32>>, String> {
    let path = root.join(format!("{table}.csv"));
    let mut reader =
        csv::Reader::from_path(&path).map_err(|error| format!("{}: {error}", path.display()))?;
    let headers = reader
        .headers()
        .map_err(|error| format!("{table} headers: {error}"))?
        .clone();
    let indices = fields
        .iter()
        .map(|field| {
            headers
                .iter()
                .position(|header| header == *field)
                .ok_or_else(|| format!("{table} missing column{field}"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let rows = reader
        .records()
        .enumerate()
        .map(|(number, row)| {
            let row = row.map_err(|error| format!("{table} row{}: {error}", number + 2))?;
            indices
                .iter()
                .map(|&index| {
                    row[index].parse::<u32>().map_err(|error| {
                        format!(
                            "{table} row{} column{}: {error}",
                            number + 2,
                            &headers[index]
                        )
                    })
                })
                .collect::<Result<Vec<_>, _>>()
        })
        .collect::<Result<Vec<_>, _>>()?;
    if rows.is_empty() {
        return Err(format!("{table} contains no rows"));
    }
    Ok(rows)
}
