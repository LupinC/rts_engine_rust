use std::{fs, path::Path};

use anyhow::{Result, anyhow};
use crate::backend::map::data::MapData;

pub fn parse_mpr(path: &str) -> Result<MapData> {
    let text = fs::read_to_string(path)?;
    let mut map: MapData =
        serde_json::from_str(&text).map_err(|e| anyhow!("Invalid .mpr {}: {e}", path))?;
    if map.width <= 0 || map.height <= 0 {
        return Err(super::invalid_map_size(path));
    }
    map.ensure_elevations();
    Ok(map)
}

pub fn save_mpr<P: AsRef<Path>>(path: P, map: &MapData) -> Result<()> {
    let json = serde_json::to_string_pretty(map)?;
    fs::write(path, json)?;
    Ok(())
}
