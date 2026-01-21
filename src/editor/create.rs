use bevy::prelude::*;
use bevy_panorbit_camera::PanOrbitCamera;
use std::path::PathBuf;

use crate::editor::*;
/// 新建空编辑器
pub fn create_blank_editor(
    commands: Commands,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<StandardMaterial>>,
) {
    create(commands, meshes, materials, None);
}

pub fn create(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    ron_file: Option<PathBuf>,
) {
    let editor_scene = commands
        .spawn((
            Editor {
                path: ron_file.clone(),
                is_dirty: false,
            },
            Transform::default(),
            GlobalTransform::default(),
            Visibility::default(),
            InheritedVisibility::default(),
        ))
        .with_children(|editor_scene| {
            // 3D 相机
            editor_scene.spawn(((
                EditorPart, // 标记为编辑器的零件
                Transform::from_translation(Vec3::new(0.0, 1.5, 5.0)),
                PanOrbitCamera {
                    button_orbit: MouseButton::Middle,
                    button_pan: MouseButton::Middle,
                    modifier_orbit: Some(KeyCode::ShiftLeft),
                    ..default()
                },
            ),));
            // 灯光
            editor_scene.spawn(((
                EditorPart, // 标记为编辑器的零件
                DirectionalLight {
                    illuminance: 10000.0,
                    ..default()
                },
                Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.5, 0.5, 0.0)),
            ),));
        })
        .id();

    if ron_file.is_none() {
        // 立方体
        let cube = commands
            .spawn((
                Mesh3d(meshes.add(Cuboid::new(2.0, 2.0, 2.0))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: Color::srgb(0.8, 0.7, 0.6),
                    ..default()
                })),
                Transform::from_xyz(0.0, 1.0, 0.0),
            ))
            .id();

        // 球体
        let sphere = commands
            .spawn((
                Mesh3d(meshes.add(Sphere::new(1.0))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: Color::srgb(0.5, 0.4, 0.3),
                    ..default()
                })),
                Transform::from_xyz(3.0, 1.0, 0.0),
            ))
            .id();

        // 地面
        let ground = commands
            .spawn((
                Mesh3d(meshes.add(Plane3d::default().mesh().size(20.0, 20.0))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: Color::srgb(0.3, 0.5, 0.3),
                    ..default()
                })),
                Transform::from_xyz(0.0, 0.0, 0.0),
            ))
            .id();

        commands
            .entity(editor_scene)
            .add_children(&[cube, sphere, ground]);
    }
    else{
        // 后续从 ron 文件加载场景
        todo!();
    }
}
