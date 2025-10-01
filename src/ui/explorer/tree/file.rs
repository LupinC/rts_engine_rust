use bevy_egui::egui;

use crate::backend::Node;

use super::{ICON_SPACE, INDENT_PER_LEVEL, ROW_HEIGHT};

pub(super) struct FileRow {
    pub response: egui::Response,
    pub text_rect: egui::Rect,
}

pub(super) fn paint_file_row(ui: &mut egui::Ui, node: &Node, depth: usize) -> FileRow {
    let (rect, response) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), ROW_HEIGHT),
        egui::Sense::click(),
    );

    if response.hovered() {
        ui.painter().rect_filled(
            rect,
            2.0,
            egui::Color32::from_rgba_unmultiplied(255, 255, 255, 8),
        );
    }

    let indent = INDENT_PER_LEVEL * depth as f32 + ICON_SPACE;
    let mut cursor = rect.left_top() + egui::vec2(indent, 0.0);
    let file_icon = "📄";
    let color = egui::Color32::from_gray(200);

    ui.painter().text(
        cursor + egui::vec2(0.0, 3.0),
        egui::Align2::LEFT_TOP,
        file_icon,
        egui::FontId::monospace(14.0),
        color,
    );
    cursor.x += ICON_SPACE;

    ui.painter().text(
        cursor + egui::vec2(0.0, 3.0),
        egui::Align2::LEFT_TOP,
        &node.name,
        egui::FontId::proportional(14.0),
        egui::Color32::from_gray(230),
    );

    let text_rect = egui::Rect::from_min_max(
        egui::pos2(cursor.x, rect.top() + 2.0),
        egui::pos2(rect.right() - 4.0, rect.bottom() - 2.0),
    );

    FileRow {
        response,
        text_rect,
    }
}
