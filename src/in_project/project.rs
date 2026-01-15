use bevy::prelude::*;
use std::path::PathBuf;

#[derive(Resource)]
pub struct Project {
    pub path: PathBuf,
    pub name: String,
}

impl Project {
    pub fn new(path: PathBuf) -> Self {
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("未知项目名")
            .to_string();

        Self { path, name }
    }
}
