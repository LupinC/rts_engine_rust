use bevy_egui::egui;

use crate::backend::Tool;

pub(super) fn draw_iso_grid(
    painter: &egui::Painter,
    origin: egui::Pos2,
    w_tiles: i32,
    h_tiles: i32,
    tile_w: f32,
    tile_h: f32,
) {
    let grid = egui::Color32::from_rgba_unmultiplied(0, 0, 0, 80);
    for y in 0..=h_tiles {
        let a = cell_to_screen(0.0, y as f32, tile_w, tile_h, origin);
        let b = cell_to_screen(w_tiles as f32, y as f32, tile_w, tile_h, origin);
        painter.add(egui::Shape::line_segment(
            [a, b],
            egui::Stroke::new(1.0, grid),
        ));
    }
    for x in 0..=w_tiles {
        let a = cell_to_screen(x as f32, 0.0, tile_w, tile_h, origin);
        let b = cell_to_screen(x as f32, h_tiles as f32, tile_w, tile_h, origin);
        painter.add(egui::Shape::line_segment(
            [a, b],
            egui::Stroke::new(1.0, grid),
        ));
    }
}

pub(super) fn draw_marker_circle(
    painter: &egui::Painter,
    cx_center: f32,
    cy_center: f32,
    tile_w: f32,
    tile_h: f32,
    origin: egui::Pos2,
    color: egui::Color32,
) {
    let c = cell_to_screen(cx_center, cy_center, tile_w, tile_h, origin);
    painter.circle_filled(c, (tile_h * 0.4) as f32, color);
    painter.circle_stroke(
        c,
        (tile_h * 0.4) as f32,
        egui::Stroke::new(1.5, egui::Color32::BLACK),
    );
}

pub(super) fn draw_marker_triangle(
    painter: &egui::Painter,
    cx_center: f32,
    cy_center: f32,
    tile_w: f32,
    tile_h: f32,
    origin: egui::Pos2,
    color: egui::Color32,
) {
    let size = tile_h * 0.55;
    let c = cell_to_screen(cx_center, cy_center, tile_w, tile_h, origin);
    let p1 = egui::pos2(c.x, c.y - size * 0.8);
    let p2 = egui::pos2(c.x - size * 0.7, c.y + size * 0.5);
    let p3 = egui::pos2(c.x + size * 0.7, c.y + size * 0.5);
    painter.add(egui::Shape::convex_polygon(
        vec![p1, p2, p3],
        color,
        egui::Stroke::new(1.5, egui::Color32::BLACK),
    ));
}

pub(super) fn draw_marker_diamond(
    painter: &egui::Painter,
    cx_center: f32,
    cy_center: f32,
    tile_w: f32,
    tile_h: f32,
    origin: egui::Pos2,
    color: egui::Color32,
) {
    let d = diamond_points(cx_center, cy_center, tile_w, tile_h, origin);
    painter.add(egui::Shape::convex_polygon(
        d.to_vec(),
        color,
        egui::Stroke::new(1.5, egui::Color32::BLACK),
    ));
}

pub(super) fn draw_marker_kind(
    painter: &egui::Painter,
    kind: Tool,
    cx_center: f32,
    cy_center: f32,
    tile_w: f32,
    tile_h: f32,
    origin: egui::Pos2,
) {
    match kind {
        Tool::Spawn => draw_marker_circle(
            painter,
            cx_center,
            cy_center,
            tile_w,
            tile_h,
            origin,
            egui::Color32::from_rgb(60, 220, 120),
        ),
        Tool::Resource => draw_marker_diamond(
            painter,
            cx_center,
            cy_center,
            tile_w,
            tile_h,
            origin,
            egui::Color32::from_rgb(245, 210, 60),
        ),
        Tool::Unit => draw_marker_triangle(
            painter,
            cx_center,
            cy_center,
            tile_w,
            tile_h,
            origin,
            egui::Color32::from_rgb(60, 200, 245),
        ),
        Tool::Select => {}
    }
}

pub(super) fn cell_to_screen(
    cx: f32,
    cy: f32,
    tile_w: f32,
    tile_h: f32,
    origin: egui::Pos2,
) -> egui::Pos2 {
    let x = (cx - cy) * (tile_w * 0.5);
    let y = (cx + cy) * (tile_h * 0.5);
    egui::pos2(origin.x + x, origin.y + y)
}

pub(super) fn screen_to_cell(
    px: f32,
    py: f32,
    tile_w: f32,
    tile_h: f32,
    origin: egui::Pos2,
) -> (f32, f32) {
    let dx = px - origin.x;
    let dy = py - origin.y;
    let a = dx / (tile_w * 0.5);
    let b = dy / (tile_h * 0.5);
    let cx = (a + b) * 0.5;
    let cy = (b - a) * 0.5;
    (cx, cy)
}

pub(super) fn pick_cell(
    mouse: egui::Pos2,
    origin: egui::Pos2,
    tile_w: f32,
    tile_h: f32,
    w_tiles: i32,
    h_tiles: i32,
) -> Option<(i32, i32)> {
    let (cx, cy) = screen_to_cell(mouse.x, mouse.y, tile_w, tile_h, origin);
    let sx = cx.floor() as i32;
    let sy = cy.floor() as i32;
    if sx >= 0 && sy >= 0 && sx < w_tiles && sy < h_tiles {
        Some((sx, sy))
    } else {
        None
    }
}

pub(super) fn diamond_points(
    cx_center: f32,
    cy_center: f32,
    tile_w: f32,
    tile_h: f32,
    origin: egui::Pos2,
) -> [egui::Pos2; 4] {
    let c = cell_to_screen(cx_center, cy_center, tile_w, tile_h, origin);
    [
        egui::pos2(c.x - tile_w * 0.5, c.y),
        egui::pos2(c.x, c.y - tile_h * 0.5),
        egui::pos2(c.x + tile_w * 0.5, c.y),
        egui::pos2(c.x, c.y + tile_h * 0.5),
    ]
}
