use bevy::prelude::*;

use crate::editor::Editor;

pub fn dispose_system(mut commands: Commands, editor_query: Query<Entity, With<Editor>>) {
    for editor in &editor_query {
        // 级联删除：相机及其所有子实体（立方体、球体、灯光等）
        commands.entity(editor).despawn();
        println!("dispose");
    }
}
