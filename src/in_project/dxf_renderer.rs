use bevy::prelude::*;
use bevy_panorbit_camera::PanOrbitCamera;
use dxf::Drawing;
use dxf::entities::EntityType;
use std::path::PathBuf;

use crate::editor::Editor;

/// DXF 加载消息
#[derive(Message, Debug)]
pub struct LoadDxfMessage {
    pub path: PathBuf,
}

/// CAD 实体类型
#[derive(Debug, Clone, PartialEq)]
pub enum CadEntityType {
    Line,
    Circle,
    Arc,
    Polyline,
}

/// CAD 实体组件 - 标记和存储CAD实体信息
#[derive(Component, Debug)]
pub struct CadEntity {
    pub entity_type: CadEntityType,
    pub layer: String,
    pub color: Color,
    pub selectable: bool,
}

/// 线段实体数据
#[derive(Component, Debug)]
pub struct LineEntity {
    pub start: Vec3,
    pub end: Vec3,
}

/// 圆形实体数据
#[derive(Component, Debug)]
pub struct CircleEntity {
    pub center: Vec3,
    pub radius: f32,
}

/// 弧形实体数据
#[derive(Component, Debug)]
pub struct ArcEntity {
    pub center: Vec3,
    pub radius: f32,
    pub start_angle: f32,
    pub end_angle: f32,
}

/// 多段线实体数据
#[derive(Component, Debug)]
pub struct PolylineEntity {
    pub vertices: Vec<Vec3>,
    pub closed: bool,
}

/// 存储已加载的 DXF 数据用于 Gizmos 绘制
#[derive(Resource, Default)]
pub struct DxfDrawData {
    pub lines: Vec<(Vec3, Vec3, Color)>,
    pub circles: Vec<(Vec3, f32, Color)>,
    pub arcs: Vec<(Vec3, f32, f32, f32, Color)>,
    pub polylines: Vec<(Vec<Vec3>, bool, Color)>,
}

impl DxfDrawData {
    pub fn clear(&mut self) {
        self.lines.clear();
        self.circles.clear();
        self.arcs.clear();
        self.polylines.clear();
    }
}

/// DXF 加载系统 - 解析 DXF 并创建 CAD 实体
pub fn dxf_load_system(
    mut messages: MessageReader<LoadDxfMessage>,
    mut commands: Commands,
    mut draw_data: ResMut<DxfDrawData>,
    children_query: Query<&Children, With<Editor>>,
    editor_query: Query<Entity, With<Editor>>,
    // 用 Has<T> 或 Query 过滤相机和灯光
    to_keep_query: Query<(Has<PanOrbitCamera>, Has<DirectionalLight>)>,
) {
    for message in messages.read() {
        draw_data.clear();
        
        // 清理当前的 CAD 实体
        for children in &children_query {
            for &child in children {
                // 检查这个子实体是否需要保留
                if let Ok((is_camera, is_light)) = to_keep_query.get(child) {
                    if !is_camera && !is_light {
                        // 如果既不是相机也不是灯光，则删除
                        commands.entity(child).despawn();
                    }
                }
            }
        }

        let drawing = match Drawing::load_file(&message.path) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("加载DXF失败: {}", e);
                continue;
            }
        };

        // 获取 Editor 实体
        let editor_entity = if let Some(entity) = editor_query.iter().next() {
            entity
        } else {
            eprintln!("未找到 Editor 实体");
            continue;
        };

        let line_color = Color::WHITE;
        let circle_color = Color::srgb(0.0, 1.0, 1.0);
        let arc_color = Color::srgb(1.0, 1.0, 0.0);
        let poly_color = Color::srgb(0.0, 1.0, 0.0);

        let mut line_count = 0;
        let mut circle_count = 0;
        let mut arc_count = 0;
        let mut polyline_count = 0;

        for ent in drawing.entities() {
            match &ent.specific {
                EntityType::Line(line) => {
                    let start = Vec3::new(line.p1.x as f32, line.p1.z as f32, line.p1.y as f32);
                    let end = Vec3::new(line.p2.x as f32, line.p2.z as f32, line.p2.y as f32);
                    
                    // 添加到绘制数据
                    draw_data.lines.push((start, end, line_color));
                    
                    // 创建CAD实体（不可见，但可选择）
                    let line_entity = commands.spawn((
                        Transform::from_translation((start + end) * 0.5),
                        Visibility::Hidden, // 隐藏，因为我们用Gizmos绘制
                        CadEntity {
                            entity_type: CadEntityType::Line,
                            layer: ent.common.layer.clone(),
                            color: line_color,
                            selectable: true,
                        },
                        LineEntity { start, end },
                    )).id();
                    
                    commands.entity(editor_entity).add_child(line_entity);
                    line_count += 1;
                }
                EntityType::Circle(c) => {
                    let center = Vec3::new(c.center.x as f32, c.center.z as f32, c.center.y as f32);
                    let radius = c.radius as f32;
                    
                    // 添加到绘制数据
                    draw_data.circles.push((center, radius, circle_color));
                    
                    // 创建CAD实体
                    let circle_entity = commands.spawn((
                        Transform::from_translation(center),
                        Visibility::Hidden,
                        CadEntity {
                            entity_type: CadEntityType::Circle,
                            layer: ent.common.layer.clone(),
                            color: circle_color,
                            selectable: true,
                        },
                        CircleEntity { center, radius },
                    )).id();
                    
                    commands.entity(editor_entity).add_child(circle_entity);
                    circle_count += 1;
                }
                EntityType::Arc(arc) => {
                    let center = Vec3::new(
                        arc.center.x as f32,
                        arc.center.z as f32,
                        arc.center.y as f32,
                    );
                    let radius = arc.radius as f32;
                    let start_angle = arc.start_angle.to_radians() as f32;
                    let end_angle = arc.end_angle.to_radians() as f32;
                    
                    // 添加到绘制数据
                    draw_data.arcs.push((center, radius, start_angle, end_angle, arc_color));
                    
                    // 创建CAD实体
                    let arc_entity = commands.spawn((
                        Transform::from_translation(center),
                        Visibility::Hidden,
                        CadEntity {
                            entity_type: CadEntityType::Arc,
                            layer: ent.common.layer.clone(),
                            color: arc_color,
                            selectable: true,
                        },
                        ArcEntity {
                            center,
                            radius,
                            start_angle,
                            end_angle,
                        },
                    )).id();
                    
                    commands.entity(editor_entity).add_child(arc_entity);
                    arc_count += 1;
                }
                EntityType::LwPolyline(pl) => {
                    if pl.vertices.len() >= 2 {
                        let vertices: Vec<Vec3> = pl
                            .vertices
                            .iter()
                            .map(|v| Vec3::new(v.x as f32, 0.0, v.y as f32))
                            .collect();
                        let closed = pl.is_closed();
                        
                        // 添加到绘制数据
                        draw_data.polylines.push((vertices.clone(), closed, poly_color));
                        
                        // 计算中心点
                        let center = vertices.iter().fold(Vec3::ZERO, |acc, v| acc + *v) / vertices.len() as f32;
                        
                        // 创建CAD实体
                        let polyline_entity = commands.spawn((
                            Transform::from_translation(center),
                            Visibility::Hidden,
                            CadEntity {
                                entity_type: CadEntityType::Polyline,
                                layer: ent.common.layer.clone(),
                                color: poly_color,
                                selectable: true,
                            },
                            PolylineEntity { vertices, closed },
                        )).id();
                        
                        commands.entity(editor_entity).add_child(polyline_entity);
                        polyline_count += 1;
                    }
                }
                EntityType::Polyline(pl) => {
                    let verts: Vec<_> = pl.vertices().collect();
                    if verts.len() >= 2 {
                        let vertices: Vec<Vec3> = verts
                            .iter()
                            .map(|v| {
                                Vec3::new(
                                    v.location.x as f32,
                                    v.location.z as f32,
                                    v.location.y as f32,
                                )
                            })
                            .collect();
                        let closed = pl.is_closed();
                        
                        // 添加到绘制数据
                        draw_data.polylines.push((vertices.clone(), closed, poly_color));
                        
                        // 计算中心点
                        let center = vertices.iter().fold(Vec3::ZERO, |acc, v| acc + *v) / vertices.len() as f32;
                        
                        // 创建CAD实体
                        let polyline_entity = commands.spawn((
                            Transform::from_translation(center),
                            Visibility::Hidden,
                            CadEntity {
                                entity_type: CadEntityType::Polyline,
                                layer: ent.common.layer.clone(),
                                color: poly_color,
                                selectable: true,
                            },
                            PolylineEntity { vertices, closed },
                        )).id();
                        
                        commands.entity(editor_entity).add_child(polyline_entity);
                        polyline_count += 1;
                    }
                }
                _ => {}
            }
        }
        
        println!(
            "DXF 加载完成: {:?}, 线段: {}, 圆: {}, 弧: {}, 多段线: {}",
            message.path, line_count, circle_count, arc_count, polyline_count
        );
    }
}

/// DXF Gizmos 绘制系统 - 每帧绘制
pub fn dxf_gizmos_system(mut gizmos: Gizmos, draw_data: Res<DxfDrawData>) {
    // 绘制线段
    for (start, end, color) in &draw_data.lines {
        gizmos.line(*start, *end, *color);
    }

    // 绘制圆（在 XZ 平面）
    for (center, radius, color) in &draw_data.circles {
        // Gizmos 的 circle 需要指定法向量，DXF 通常在 XZ 平面
        gizmos.circle(
            Isometry3d::new(*center, Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)),
            *radius,
            *color,
        );
    }

    // 绘制弧
    for (center, radius, start_angle, end_angle, color) in &draw_data.arcs {
        draw_arc(
            &mut gizmos,
            *center,
            *radius,
            *start_angle,
            *end_angle,
            *color,
        );
    }

    // 绘制多段线
    for (points, closed, color) in &draw_data.polylines {
        if points.len() < 2 {
            continue;
        }
        for i in 0..points.len() - 1 {
            gizmos.line(points[i], points[i + 1], *color);
        }
        if *closed && points.len() > 2 {
            gizmos.line(points[points.len() - 1], points[0], *color);
        }
    }
}

/// 绘制弧线（分段近似）
fn draw_arc(
    gizmos: &mut Gizmos,
    center: Vec3,
    radius: f32,
    start_angle: f32,
    end_angle: f32,
    color: Color,
) {
    let segments = 32;
    let mut diff = end_angle - start_angle;
    if diff <= 0.0 {
        diff += std::f32::consts::TAU;
    }
    let step = diff / segments as f32;

    for i in 0..segments {
        let a1 = start_angle + i as f32 * step;
        let a2 = start_angle + (i + 1) as f32 * step;
        let p1 = center + Vec3::new(radius * a1.cos(), 0.0, radius * a1.sin());
        let p2 = center + Vec3::new(radius * a2.cos(), 0.0, radius * a2.sin());
        gizmos.line(p1, p2, color);
    }
}