use anyhow::{anyhow, Result};
use std::fs;

/// Minimal parse for RA2/YR `.map` headers we care about:
/// - [Map] Theater=Temperate|Snow|Urban|NewUrban|Desert|Lunar (any case)
/// - [Map] Size can be either:
///     * "W,H"
///     * "X,Y,W,H"  ← common in RA2/YR (x,y offset + size in tiles)
/// - Fallback: [Header] Width / Height (if [Map] Size missing)
///
/// We are NOT decoding IsoMapPack5 yet; this is only enough to paint a theater grid.
#[derive(Debug, Clone)]
pub struct MapHeader {
    pub theater: Theater,
    pub width: i32,
    pub height: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Theater {
    Temperate,
    Snow,
    Urban,
    NewUrban,
    Desert,
    Lunar,
    Unknown,
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

pub fn parse_map_header(path: &str) -> Result<MapHeader> {
    let text = fs::read_to_string(path)?;

    // State
    let mut in_map = false;
    let mut in_header = false;

    // Parsed values
    let mut theater: Option<Theater> = None;
    let mut map_size_wh: Option<(i32, i32)> = None;   // from [Map] Size
    let mut header_wh: Option<(i32, i32)> = None;     // from [Header] Width/Height

    for raw in text.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with(';') { continue; }

        // Section switches
        if line.starts_with('[') && line.ends_with(']') {
            let section = &line[1..line.len()-1];
            in_map = section.eq_ignore_ascii_case("Map");
            in_header = section.eq_ignore_ascii_case("Header");
            continue;
        }

        // Key = Value
        if let Some((k, v)) = line.split_once('=') {
            let key = k.trim();
            let val = v.trim();

            if in_map {
                match key.to_ascii_lowercase().as_str() {
                    "theater" => theater = Some(Theater::from_str(val)),
                    "size" => {
                        // Accept "W,H" or "X,Y,W,H". Use the LAST two numbers as width/height.
                        let nums: Vec<_> = val
                            .split(',')
                            .map(|s| s.trim().parse::<i32>())
                            .collect();

                        // All must parse; otherwise ignore this key gracefully.
                        if nums.iter().all(|r| r.is_ok()) {
                            let parsed: Vec<i32> = nums.into_iter().map(|r| r.unwrap()).collect();
                            match parsed.len() {
                                2 => map_size_wh = Some((parsed[0], parsed[1])),
                                4 => map_size_wh = Some((parsed[2], parsed[3])),
                                _ => { /* ignore weird sizes */ }
                            }
                        }
                    }
                    _ => {}
                }
            } else if in_header {
                match key.to_ascii_lowercase().as_str() {
                    "width" => {
                        // Capture width; height may come later (or earlier)
                        let w = val.parse::<i32>().ok();
                        if let Some(w) = w {
                            header_wh = Some((w, header_wh.map(|(_,h)| h).unwrap_or_default()));
                        }
                    }
                    "height" => {
                        let h = val.parse::<i32>().ok();
                        if let Some(h) = h {
                            header_wh = Some((header_wh.map(|(w,_)| w).unwrap_or_default(), h));
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    // Normalize header_wh if one of width/height was missing when set in two steps
    if let Some((w, h)) = header_wh {
        if w == 0 || h == 0 {
            // leave it; final validation happens below with fallbacks
        }
    }

    // Prefer [Map] Size; fallback to [Header] Width/Height; else defaults.
    let (mut width, mut height) = if let Some((w, h)) = map_size_wh {
        (w, h)
    } else if let Some((w, h)) = header_wh {
        (w, h)
    } else {
        (64, 64)
    };

    // Final sanity: width/height must be > 0. If not, try other fallbacks before failing.
    if width <= 0 || height <= 0 {
        if let Some((w, h)) = header_wh {
            width = w;
            height = h;
        }
    }
    if width <= 0 || height <= 0 {
        return Err(anyhow!("Invalid size (width/height <= 0) in {}", path));
    }

    let theater = theater.unwrap_or(Theater::Unknown);
    Ok(MapHeader { theater, width, height })
}
