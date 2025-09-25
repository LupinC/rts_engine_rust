use bevy::prelude::*;

/// Menu action: open/close a folder via OS dialog.
#[derive(Event, Debug, Clone)]
pub enum OpenFolder {
    Pick,
    Close,
}

/// Explorer action: user clicked a file we may want to open.
/// Only `.map` will be handled for now; others are ignored.
#[derive(Event, Debug, Clone)]
pub struct OpenMap {
    pub path: String, // absolute or normalized path from Node.id
}
