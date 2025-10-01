use anyhow::{Result, anyhow};

use super::data::MapData;

mod legacy;
mod mpr;

pub use mpr::save_mpr;

pub fn parse_map(path: &str) -> Result<MapData> {
    if path.to_ascii_lowercase().ends_with(".mpr") {
        mpr::parse_mpr(path)
    } else {
        legacy::parse_legacy_ini_map(path)
    }
}
/// Shared helper for parse modules.
pub(super) fn invalid_map_size(path: &str) -> anyhow::Error {
    anyhow!("Invalid map size in {}", path)
}
