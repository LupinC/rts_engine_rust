use bevy::prelude::*;

/// Menu action: open/close a folder via OS dialog.
#[derive(Event, Debug, Clone)]
pub enum OpenFolder {
    Pick,
    Close,
}

/// Menu action: create a brand-new project scaffold at a user-selected location.
#[derive(Event, Debug, Clone)]
pub struct CreateProject;

/// Explorer action: user clicked a file we may want to open.
/// Only `.mpr` will be handled for now; others are ignored.
#[derive(Event, Debug, Clone)]
pub struct OpenMap {
    pub path: String, // absolute or normalized path from Node.id
}

/// Explorer context-menu commands (new file/folder, etc.).
#[derive(Event, Debug, Clone)]
pub enum ExplorerCommand {
    NewFile { parent: String },
    NewFolder { parent: String },
}
