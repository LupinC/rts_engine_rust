use bevy::prelude::Vec2;
use bevy_egui::egui;

use crate::{
    backend::{map::IsoStaggered, Tool},
};

pub(super) fn draw_iso_grid(
    painter: &egui::Painter,
    iso: &IsoStaggered,
    w_tiles: i32,
    h_tiles: i32,
) {
    let stroke = egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(40, 40, 40, 200));
    for y in 0..h_tiles {
        for x in 0..w_tiles {
            let outline = tile_outline_points(iso, x, y);
            painter.add(egui::Shape::closed_line(outline.to_vec(), stroke));
        }
    }
}

pub(super) fn draw_marker_circle(
    painter: &egui::Painter,
    iso: &IsoStaggered,
    x: i32,
    y: i32,
    color: egui::Color32,
) {
    let center = tile_center(iso, x, y);
    let radius = iso.tile.h * 0.35;
    painter.circle_filled(center, radius, color);
    painter.circle_stroke(center, radius, egui::Stroke::new(1.5, egui::Color32::BLACK));
}

pub(super) fn draw_marker_triangle(
    painter: &egui::Painter,
    iso: &IsoStaggered,
    x: i32,
    y: i32,
    color: egui::Color32,
) {
    let center = tile_center(iso, x, y);
    let size = iso.tile.h * 0.6;
    let top = egui::pos2(center.x, center.y - size * 0.55);
    let left = egui::pos2(center.x - size * 0.55, center.y + size * 0.35);
    let right = egui::pos2(center.x + size * 0.55, center.y + size * 0.35);
    painter.add(egui::Shape::convex_polygon(
        vec![top, right, left],
        color,
        egui::Stroke::new(1.5, egui::Color32::BLACK),
    ));
}

pub(super) fn draw_marker_diamond(
    painter: &egui::Painter,
    iso: &IsoStaggered,
    x: i32,
    y: i32,
    color: egui::Color32,
) {
    let mut outline = tile_outline_points(iso, x, y);
    shrink_polygon_around_center(&mut outline, 0.7);
    painter.add(egui::Shape::convex_polygon(
        outline.to_vec(),
        color,
        egui::Stroke::new(1.5, egui::Color32::BLACK),
    ));
}

pub(super) fn fill_tile(
    painter: &egui::Painter,
    iso: &IsoStaggered,
    x: i32,
    y: i32,
    color: egui::Color32,
) {
    let outline = tile_outline_points(iso, x, y);
    painter.add(egui::Shape::convex_polygon(
        outline.to_vec(),
        color,
        egui::Stroke::NONE,
    ));
}

pub(super) fn draw_marker_kind(
    painter: &egui::Painter,
    kind: Tool,
    iso: &IsoStaggered,
    x: i32,
    y: i32,
) {
    match kind {
        Tool::Spawn => draw_marker_circle(
            painter,
            iso,
            x,
            y,
            egui::Color32::from_rgb(60, 220, 120),
        ),
        Tool::Resource => draw_marker_diamond(
            painter,
            iso,
            x,
            y,
            egui::Color32::from_rgb(245, 210, 60),
        ),
        Tool::Unit => draw_marker_triangle(
            painter,
            iso,
            x,
            y,
            egui::Color32::from_rgb(60, 200, 245),
        ),
        Tool::Select => {}
    }
}

pub(super) fn pick_cell(
    mouse: egui::Pos2,
    iso: &IsoStaggered,
    w_tiles: i32,
    h_tiles: i32,
) -> Option<(i32, i32)> {
    let point = Vec2::new(mouse.x, mouse.y);
    iso.ij_from_world(point, w_tiles, h_tiles)
        .map(|(row, col)| (col, row))
}

pub(super) fn tile_outline_points(iso: &IsoStaggered, x: i32, y: i32) -> [egui::Pos2; 4] {
    let corners = iso.tile_corners(y, x);
    [
        egui::pos2(corners[0].x, corners[0].y),
        egui::pos2(corners[1].x, corners[1].y),
        egui::pos2(corners[2].x, corners[2].y),
        egui::pos2(corners[3].x, corners[3].y),
    ]
}

pub(super) fn tile_center(iso: &IsoStaggered, x: i32, y: i32) -> egui::Pos2 {
    let center = iso.world_center(y, x);
    egui::pos2(center.x, center.y)
}

fn shrink_polygon_around_center(poly: &mut [egui::Pos2; 4], factor: f32) {
    let cx = poly.iter().map(|p| p.x).sum::<f32>() / poly.len() as f32;
    let cy = poly.iter().map(|p| p.y).sum::<f32>() / poly.len() as f32;
    for p in poly.iter_mut() {
        p.x = cx + (p.x - cx) * factor;
        p.y = cy + (p.y - cy) * factor;
    }
}
