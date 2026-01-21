mod file_tree;
pub use file_tree::FileTree;
mod in_project_ui;
pub use in_project_ui::in_project_ui_system;
mod project;
pub use project::Project;
mod focus_change;
pub use focus_change::focus_change_system;
mod dxf_renderer;
pub use dxf_renderer::{dxf_load_system, dxf_gizmos_system, LoadDxfMessage, DxfDrawData, CadEntity, CadEntityType, LineEntity, CircleEntity, ArcEntity, PolylineEntity};

use bevy::prelude::*;
use bevy_egui::EguiPrimaryContextPass;
use bevy_panorbit_camera::PanOrbitCameraPlugin;

use crate::editor;
use crate::state::AppState;

pub struct InProjectPlugin;

impl Plugin for InProjectPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(PanOrbitCameraPlugin)
            .add_message::<LoadDxfMessage>()
            .init_resource::<DxfDrawData>()
            .add_systems(OnEnter(AppState::InPreject), editor::init_system)
            .add_systems(
                EguiPrimaryContextPass,
                in_project_ui_system.run_if(in_state(AppState::InPreject)),
            )
            .add_systems(
                Update,
                (focus_change_system, dxf_load_system, dxf_gizmos_system)
                    .run_if(in_state(AppState::InPreject)),
            )
            .add_systems(OnExit(AppState::InPreject), editor::dispose_system);
    }
}
