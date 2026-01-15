use bevy::prelude::*;
use bevy_egui::*;

use crate::{in_project::InProjectPlugin, main_menu::MainMenuPlugin};
mod editor;
mod font_system;
mod in_project;
mod main_menu;
mod state;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "开源Cad".into(),
                name: Some("开源Cad".into()),
                resolution: (1280, 720).into(),
                ..default()
            }),
            ..default()
        }))
        .init_state::<state::AppState>()
        .add_systems(Startup, set_ui_camera) // ui相机
        .add_systems(
            EguiPrimaryContextPass,
            font_system::set_heiti.run_if(run_once), // 设置黑体，只运行一次
        )
        .add_plugins((
            EguiPlugin::default(), // egui插件
            MainMenuPlugin, // 主菜单
            InProjectPlugin, // 项目中
        ))
        .run();
}

fn set_ui_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Camera {
            order: 999,
            clear_color: Color::NONE.into(),
            ..default()
        },
    ));
}
