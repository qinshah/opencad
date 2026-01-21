use bevy::prelude::*;
use bevy_panorbit_camera::PanOrbitCamera;

use crate::editor::Editor;

pub fn init_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands
        .spawn((
            Editor,
            Transform::default(),
            GlobalTransform::default(),
            Visibility::default(),
            InheritedVisibility::default(),
        ))
        .with_children(|editor| {
            // 3D 相机
            editor.spawn((
                Transform::from_translation(Vec3::new(0.0, 1.5, 5.0)),
                PanOrbitCamera {
                    button_orbit: MouseButton::Middle,
                    button_pan: MouseButton::Middle,
                    modifier_orbit: Some(KeyCode::ShiftLeft),
                    ..default()
                },
            ));
            // 灯光
            editor.spawn((
                DirectionalLight {
                    illuminance: 10000.0,
                    ..default()
                },
                Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.5, 0.5, 0.0)),
            ));
            // 立方体
            editor.spawn((
                Mesh3d(meshes.add(Cuboid::new(2.0, 2.0, 2.0))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: Color::srgb(0.8, 0.7, 0.6),
                    ..default()
                })),
                Transform::from_xyz(0.0, 1.0, 0.0),
            ));
            // 球体
            editor.spawn((
                Mesh3d(meshes.add(Sphere::new(1.0))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: Color::srgb(0.5, 0.4, 0.3),
                    ..default()
                })),
                Transform::from_xyz(3.0, 1.0, 0.0),
            ));
            // 地面
            editor.spawn((
                Mesh3d(meshes.add(Plane3d::default().mesh().size(20.0, 20.0))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: Color::srgb(0.3, 0.5, 0.3),
                    ..default()
                })),
                Transform::from_xyz(0.0, 0.0, 0.0),
            ));
        });
}
