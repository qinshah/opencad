pub mod init;
use bevy::ecs::component::Component;
pub use init::init_system;

pub mod dispose;
pub use dispose::dispose_system;

#[derive(Component)]
pub struct Editor; // 用于清理的标记
