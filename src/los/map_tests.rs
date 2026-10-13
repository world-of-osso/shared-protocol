use super::{LosMap, LosTile};
use bevy::math::Vec3;

struct AssetCache(std::path::PathBuf);

impl AssetCache {
    fn create() -> Self {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "shared-los-preparation-{}-{nonce}",
            std::process::id()
        ));
        std::fs::create_dir(&path).unwrap();
        Self(path)
    }

    fn path(&self) -> &std::path::Path {
        &self.0
    }
}

impl Drop for AssetCache {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.0).unwrap();
    }
}

#[test]
fn los_worker_region_queries_use_prepared_tiles_after_disk_changes() {
    let dir = AssetCache::create();
    let path = dir.path().join("32_32.los");
    let tile = LosTile::build(
        32,
        32,
        vec![
            Vec3::new(-4.0, 12.0, -4.0),
            Vec3::new(4.0, 12.0, -4.0),
            Vec3::new(0.0, 12.0, 4.0),
        ],
        vec![[0, 1, 2]],
    );
    std::fs::write(&path, tile.to_bytes()).unwrap();
    let mut prepared = LosMap::new(dir.path());
    prepared
        .preload_tiles((31..=33).flat_map(|row| (31..=33).map(move |col| (row, col))))
        .unwrap();
    let mut live = LosMap::new(dir.path());
    live.merge_preloaded(prepared);
    std::fs::write(&path, b"invalid now").unwrap();
    std::fs::write(
        dir.path().join("31_31.los"),
        b"was absent during preparation",
    )
    .unwrap();
    assert_eq!(
        live.floor_height(Vec3::new(0.0, 13.0, 0.0)).unwrap(),
        Some(12.0)
    );
    assert!(
        !live
            .clear(Vec3::new(0.0, 15.0, 0.0), Vec3::new(0.0, 11.0, 0.0))
            .unwrap()
    );
}

#[test]
fn los_worker_region_absence_is_ready_but_corruption_is_an_error() {
    let dir = AssetCache::create();
    let mut absent = LosMap::new(dir.path());
    absent.preload_tiles([(32, 32)]).unwrap();
    std::fs::write(dir.path().join("32_32.los"), b"invalid").unwrap();
    let mut corrupt = LosMap::new(dir.path());
    assert!(
        corrupt
            .preload_tiles([(32, 32)])
            .unwrap_err()
            .contains("32_32.los")
    );
}
