mod main_menu_ui;
mod project_manager;

pub use main_menu_ui::{ProjectListState, main_menu_ui_system};

use bevy::prelude::*;
use bevy_egui::*;

use crate::state::AppState;

pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ProjectListState>().add_systems(
            EguiPrimaryContextPass,
            main_menu_ui_system.run_if(in_state(AppState::MainMenu)),
        );
    }
}
