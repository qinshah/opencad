mod current_project;
mod in_project_ui;

use bevy::prelude::*;
use bevy_egui::EguiPrimaryContextPass;
pub use current_project::CurrentProject;
pub use in_project_ui::in_project_ui_system;

use crate::state::AppState;

pub struct InProjectPlugin;

impl Plugin for InProjectPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            EguiPrimaryContextPass,
            in_project_ui_system.run_if(in_state(AppState::InPreject)),
        );
    }
}
