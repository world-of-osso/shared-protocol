use shared::guild_creation::GuildTabardDesign;
use shared::guild_tabards::GuildTabardCatalog;

#[test]
fn guild_tabard_catalog_resolves_installed_colors_components_and_rejects_invalid_ids() {
    let root =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/guild-tabards-70391");
    let catalog = GuildTabardCatalog::load(&root).unwrap();
    let design = GuildTabardDesign {
        icon: 1,
        icon_color: 2,
        border: 3,
        border_color: 4,
        background_color: 5,
    };
    catalog.validate(design).unwrap();
    let textures = catalog.textures(design, 0).unwrap();
    assert_eq!(textures.upper, [180071, 180437, 180273]);
    assert_eq!(textures.lower, [180070, 180436, 180272]);
    assert_eq!(
        catalog.emblem_colors[&0],
        [103, 0, 33],
        "CSV Blue precedes Green, RGB does not"
    );
    for invalid in [
        GuildTabardDesign {
            icon_color: 255,
            ..design
        },
        GuildTabardDesign {
            border_color: 255,
            ..design
        },
        GuildTabardDesign {
            background_color: 255,
            ..design
        },
        GuildTabardDesign {
            icon: 255,
            ..design
        },
        GuildTabardDesign {
            border: 255,
            ..design
        },
    ] {
        assert!(catalog.validate(invalid).is_err(), "{invalid:?}");
    }
    assert!(catalog.textures(design, 255).is_err());
    assert!(GuildTabardCatalog::load(&root.join("missing")).is_err());
}
