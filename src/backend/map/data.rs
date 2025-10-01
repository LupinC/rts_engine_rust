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
    MapData {
        theater: Theater::Temperate,
        width: width.max(1),
        height: height.max(1),
        local_origin_x: 0,
        local_origin_y: 0,
        waypoints: Vec::new(),
        num_starting_points: 0,
        units: Vec::new(),
        structures: Vec::new(),
    }
}
