use bevy::ecs::system::{Local, Query};
use bevy_egui::EguiContexts;
use bevy_panorbit_camera::PanOrbitCamera;

pub fn focus_change_system(
    mut contexts: EguiContexts,
    mut query: Query<&mut PanOrbitCamera>,
    mut was_ui_focused: Local<bool>,
) {
    let ctx = contexts.ctx_mut().unwrap();
    let ui_focused = ctx.is_pointer_over_area() || ctx.wants_pointer_input();

    if *was_ui_focused != ui_focused {
        for mut camera in query.iter_mut() {
            camera.enabled = !ui_focused;
        }
        *was_ui_focused = ui_focused;
    }
}