use std::fs;

use anyhow::Result;

use crate::backend::map::data::{MapData, MapPin, Theater};

pub fn parse_legacy_ini_map(path: &str) -> Result<MapData> {
    let text = fs::read_to_string(path)?;

    let mut section = String::new();

    // Map meta
    let mut theater: Option<Theater> = None;
    let mut size_wh: Option<(i32, i32)> = None; // [Map] Size W,H or trailing W,H
    let mut local_xywh: Option<(i32, i32, i32, i32)> = None;
    let mut header_wh: Option<(i32, i32)> = None; // [Header] Width/Height fallback
    let mut start_xy: Option<(i32, i32)> = None; // [Header] StartX/StartY (global origin of local rect)

    // Waypoints
    let mut number_starting_points: usize = 0;
    let mut waypoints: Vec<(i32, i32)> = Vec::new();

    // Units/Structures
    let mut units: Vec<MapPin> = Vec::new();
    let mut structures: Vec<MapPin> = Vec::new();

    // Helper to parse "x,y" ints
    let mut push_waypoint_xy = |v: &str| {
        let parts: Vec<i32> = v
            .split(',')
            .filter_map(|s| s.trim().parse::<i32>().ok())
            .collect();
        if parts.len() == 2 {
            waypoints.push((parts[0], parts[1]));
        }
    };

    for raw in text.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with(';') {
            continue;
        }

        if line.starts_with('[') && line.ends_with(']') {
            section = line[1..line.len() - 1].to_string();
            continue;
        }

        if let Some((k, v)) = line.split_once('=') {
            let key = k.trim();
            let val = v.trim();

            match section.to_ascii_lowercase().as_str() {
                "map" => match key.to_ascii_lowercase().as_str() {
                    "theater" => theater = Some(Theater::from_str(val)),
                    "size" => {
                        let nums: Vec<i32> = val
                            .split(',')
                            .filter_map(|s| s.trim().parse().ok())
                            .collect();
                        match nums.len() {
                            2 => size_wh = Some((nums[0], nums[1])),
                            4 => size_wh = Some((nums[2], nums[3])),
                            _ => {}
                        }
                    }
                    "localsize" => {
                        let nums: Vec<i32> = val
                            .split(',')
                            .filter_map(|s| s.trim().parse().ok())
                            .collect();
                        if nums.len() == 4 {
                            local_xywh = Some((nums[0], nums[1], nums[2], nums[3]));
                        }
                    }
                    _ => {}
                },
                "header" => {
                    match key.to_ascii_lowercase().as_str() {
                        "numberstartingpoints" => {
                            number_starting_points = val.parse::<usize>().unwrap_or(0);
                        }
                        // WaypointN in [Header] are absolute map coords
                        k if k.starts_with("waypoint") => push_waypoint_xy(val),
                        "width" => {
                            let w = val.parse::<i32>().ok().unwrap_or(0);
                            header_wh = Some((w, header_wh.map(|(_, h)| h).unwrap_or_default()));
                        }
                        "height" => {
                            let h = val.parse::<i32>().ok().unwrap_or(0);
                            header_wh = Some((header_wh.map(|(w, _)| w).unwrap_or_default(), h));
                        }
                        "startx" => {
                            let x = val.parse::<i32>().ok().unwrap_or(0);
                            start_xy = Some((x, start_xy.map(|(_, y)| y).unwrap_or(0)));
                        }
                        "starty" => {
                            let y = val.parse::<i32>().ok().unwrap_or(0);
                            start_xy = Some((start_xy.map(|(x, _)| x).unwrap_or(0), y));
                        }
                        _ => {}
                    }
                }
                "waypoints" => {
                    // Support x,y form if present (ignore single integer cell encoding)
                    if key.chars().all(|c| c.is_ascii_digit()) && val.contains(',') {
                        push_waypoint_xy(val);
                    }
                }
                "units" => {
                    if let Some(pin) = parse_pin_line(val) {
                        units.push(pin);
                    }
                }
                "structures" => {
                    if let Some(pin) = parse_pin_line(val) {
                        structures.push(pin);
                    }
                }
                _ => {}
            }
        }
    }

    // Determine local rectangle size
    let (width, height) = if let Some((_, _, w, h)) = local_xywh {
        (w, h)
    } else if let Some((w, h)) = size_wh {
        (w, h)
    } else if let Some((w, h)) = header_wh {
        (w, h)
    } else {
        (64, 64)
    };

    if width <= 0 || height <= 0 {
        return Err(super::invalid_map_size(path));
    }

    // Choose origin for converting absolute coords -> local:
    // Prefer [Header].StartX/StartY (correct for absolute waypoints),
    // else fall back to [Map].LocalSize.X,Y, else 0,0.
    let (origin_x, origin_y) = if let Some((sx, sy)) = start_xy {
        (sx, sy)
    } else if let Some((lx, ly, _, _)) = local_xywh {
        (lx, ly)
    } else {
        (0, 0)
    };

    let theater = theater.unwrap_or(Theater::Unknown);

    Ok(MapData {
        theater,
        width,
        height,
        local_origin_x: origin_x,
        local_origin_y: origin_y,
        waypoints,
        num_starting_points: number_starting_points,
        units,
        structures,
    })
}

/// Parse a typical legacy CSV object line and return a pin with (x,y).
/// Strategy:
/// - Split by commas
/// - Take the **last two parseable integers** as (x,y)
/// - owner = first token (if any), kind = second token (if any)
fn parse_pin_line(csv: &str) -> Option<MapPin> {
    let parts: Vec<&str> = csv.split(',').map(|s| s.trim()).collect();

    let mut coords: Vec<i32> = Vec::new();
    for p in &parts {
        if let Ok(n) = p.parse::<i32>() {
            coords.push(n);
        }
    }
    if coords.len() < 2 {
        return None;
    }
    let x = coords[coords.len() - 2];
    let y = coords[coords.len() - 1];

    let owner = parts.get(0).map(|s| s.to_string());
    let kind = parts
        .get(1)
        .map(|s| s.to_string())
        .unwrap_or_else(|| String::from("Obj"));

    Some(MapPin { x, y, kind, owner })
}
