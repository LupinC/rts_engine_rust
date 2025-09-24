use bevy::prelude::*;

#[derive(Event, Debug, Clone)]
pub enum OpenFolder {
    Pick,   // open OS dialog and load chosen folder
    Close,  // close current project
}
