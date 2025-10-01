use bevy_egui::egui;

use crate::backend::EditorLayout;

pub(super) enum RenameOutcome {
    None,
    Commit {
        target: String,
        value: String,
        original: String,
    },
    Cancel,
}

pub(super) fn handle_rename(
    ui: &mut egui::Ui,
    layout: &mut EditorLayout,
    target_path: &str,
    _current_name: &str,
    text_rect: egui::Rect,
) -> RenameOutcome {
    if let Some(rename_state) = layout.rename.as_mut() {
        if rename_state.target_path == target_path {
            let response = ui.put(
                text_rect,
                egui::TextEdit::singleline(&mut rename_state.buffer)
                    .clip_text(false)
                    .desired_width(text_rect.width()),
            );

            if rename_state.focus_requested {
                response.request_focus();
                rename_state.focus_requested = false;
            }

            let enter = ui.input(|i| i.key_pressed(egui::Key::Enter));
            let escape = ui.input(|i| i.key_pressed(egui::Key::Escape));

            if enter {
                let trimmed = rename_state.buffer.trim().to_string();
                if trimmed.is_empty() {
                    return RenameOutcome::Cancel;
                }
                return RenameOutcome::Commit {
                    target: rename_state.target_path.clone(),
                    value: trimmed,
                    original: rename_state.original_name.clone(),
                };
            } else if escape {
                return RenameOutcome::Cancel;
            } else if response.lost_focus() && !response.has_focus() {
                return RenameOutcome::Cancel;
            }
        }
    }

    RenameOutcome::None
}
