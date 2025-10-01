use serde::{Deserialize, Serialize};

/// High-level data we render in the workspace.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapData {
    #[serde(default = "default_theater")]
    pub theater: Theater,
    pub width: i32,
    pub height: i32,
    /// Origin used to convert absolute (global) object coords -> local workspace.
    #[serde(default)]
    pub local_origin_x: i32,
    #[serde(default)]
    pub local_origin_y: i32,

    /// Per-tile elevation (z axis). Stored row-major with length = width * height.
    #[serde(default)]
    pub elevations: Vec<i32>,

    /// Waypoints from [Header].WaypointN = x,y or [Waypoints] (decoded if x,y form).
    #[serde(default)]
    pub waypoints: Vec<(i32, i32)>,
    /// First `n` waypoints are starting locations.
    #[serde(default)]
    pub num_starting_points: usize,

    /// Units & structures as generic pins (x,y + hints).
    #[serde(default)]
    pub units: Vec<MapPin>,
    #[serde(default)]
    pub structures: Vec<MapPin>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapPin {
    pub x: i32,
    pub y: i32,
    #[serde(default)]
    pub kind: String,
    #[serde(default)]
    pub owner: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Theater {
    Temperate,
    Snow,
    Urban,
    #[serde(rename = "newurban")]
    NewUrban,
    Desert,
    Lunar,
    #[serde(other)]
    Unknown,
}

impl Default for Theater {
    fn default() -> Self {
        Theater::Temperate
    }
}

fn default_theater() -> Theater {
    Theater::Temperate
}

impl Theater {
    pub fn from_str(s: &str) -> Self {
        match s.trim().to_ascii_lowercase().as_str() {
            "temperate" => Theater::Temperate,
            "snow" => Theater::Snow,
            "urban" => Theater::Urban,
            "new urban" | "newurban" => Theater::NewUrban,
            "desert" => Theater::Desert,
            "lunar" => Theater::Lunar,
            _ => Theater::Unknown,
        }
    }
}

/// Convenience for initializing a blank editor map.
pub fn blank_map(width: i32, height: i32) -> MapData {
    let mut map = MapData {
        theater: Theater::Temperate,
        width: width.max(1),
        height: height.max(1),
        local_origin_x: 0,
        local_origin_y: 0,
        elevations: Vec::new(),
        waypoints: Vec::new(),
        num_starting_points: 0,
        units: Vec::new(),
        structures: Vec::new(),
    };
    map.ensure_elevations();
    map
}

/// Load the default map template bundled with the editor (samplemap/blank.mpr).
/// Falls back to a basic 64x64 blank map if the template cannot be parsed.
pub fn default_map_template() -> MapData {
    const TEMPLATE_JSON: &str = include_str!("../../samplemap/blank.mpr");
    match serde_json::from_str::<MapData>(TEMPLATE_JSON) {
        Ok(mut map) => {
            map.ensure_elevations();
            map
        }
        Err(_) => blank_map(64, 64),
    }
}

impl MapData {
    fn tile_count(&self) -> usize {
        if self.width <= 0 || self.height <= 0 {
            0
        } else {
            (self.width as usize).saturating_mul(self.height as usize)
        }
    }

    /// Ensure the elevation buffer matches the map dimensions, padding with zeros when absent.
    pub fn ensure_elevations(&mut self) {
        let expected = self.tile_count();
        if expected == 0 {
            self.elevations.clear();
        } else if self.elevations.len() != expected {
            self.elevations.resize(expected, 0);
        }
    }

    /// Return the elevation (z) for a given local tile coordinate.
    pub fn elevation_at(&self, x: i32, y: i32) -> Option<i32> {
        if x < 0 || y < 0 || x >= self.width || y >= self.height {
            return None;
        }
        let idx = y as usize * self.width as usize + x as usize;
        self.elevations.get(idx).copied()
    }
}
