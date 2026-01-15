use bevy::prelude::*;
use bevy_egui::*;
mod font_system;
mod state;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "开源Cad".into(),
                    name: Some("开源Cad".into()),
                    resolution: (1280, 720).into(),
                    ..default()
                }),
                ..default()
            }),
            EguiPlugin::default(), // egui插件
        ))
        .init_state::<state::AppState>()
        .add_systems(Startup, set_ui_camera) // ui相机
        .add_systems(
            EguiPrimaryContextPass,
            font_system::set_heiti.run_if(run_once), // 设置黑体，只运行一次
        )
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
