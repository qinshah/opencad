use bevy::prelude::*;

use crate::{editor::Editor, in_project::DxfDrawData};

pub fn dispose_system(
    mut commands: Commands,
    editor_query: Query<Entity, With<Editor>>,
    mut draw_data: ResMut<DxfDrawData>,
) {
    draw_data.clear();
    for editor in &editor_query {
        commands.entity(editor).despawn();
    }
}
