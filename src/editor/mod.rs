pub mod create;
use bevy::ecs::component::Component;
use std::path::PathBuf;
pub use create::create_blank_editor;

pub mod dispose;
pub use dispose::dispose_system;

#[derive(Component)]
pub struct Editor {
    /// 对应的文件路径。None 表示这是一个"新打开未保存"的编辑器
    pub path: Option<PathBuf>,
    /// 标记数据是否有未保存的更改（脏标记）
    pub is_dirty: bool,
}

#[derive(Component)]
pub struct EditorPart; // 标记这是编辑器自带的“零件”（相机、灯光）