use bevy::prelude::*;
use dxf::entities::EntityType;
use dxf::Drawing;
use std::path::PathBuf;

/// DXF 加载消息
#[derive(Message, Debug)]
pub struct LoadDxfEvent {
    pub path: PathBuf,
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

/// DXF 加载系统 - 解析 DXF 并存储绘制数据
pub fn dxf_load_system(
    mut events: MessageReader<LoadDxfEvent>,
    mut draw_data: ResMut<DxfDrawData>,
) {
    for event in events.read() {
        draw_data.clear();

        let drawing = match Drawing::load_file(&event.path) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("加载DXF失败: {}", e);
                continue;
            }
        };

        let line_color = Color::WHITE;
        let circle_color = Color::srgb(0.0, 1.0, 1.0);
        let arc_color = Color::srgb(1.0, 1.0, 0.0);
        let poly_color = Color::srgb(0.0, 1.0, 0.0);

        for ent in drawing.entities() {
            match &ent.specific {
                EntityType::Line(line) => {
                    let s = Vec3::new(line.p1.x as f32, line.p1.z as f32, line.p1.y as f32);
                    let e = Vec3::new(line.p2.x as f32, line.p2.z as f32, line.p2.y as f32);
                    draw_data.lines.push((s, e, line_color));
                }
                EntityType::Circle(c) => {
                    let center = Vec3::new(c.center.x as f32, c.center.z as f32, c.center.y as f32);
                    draw_data.circles.push((center, c.radius as f32, circle_color));
                }
                EntityType::Arc(arc) => {
                    let center = Vec3::new(arc.center.x as f32, arc.center.z as f32, arc.center.y as f32);
                    let sa = arc.start_angle.to_radians() as f32;
                    let ea = arc.end_angle.to_radians() as f32;
                    draw_data.arcs.push((center, arc.radius as f32, sa, ea, arc_color));
                }
                EntityType::LwPolyline(pl) => {
                    if pl.vertices.len() >= 2 {
                        let pts: Vec<Vec3> = pl.vertices.iter()
                            .map(|v| Vec3::new(v.x as f32, 0.0, v.y as f32))
                            .collect();
                        draw_data.polylines.push((pts, pl.is_closed(), poly_color));
                    }
                }
                EntityType::Polyline(pl) => {
                    let verts: Vec<_> = pl.vertices().collect();
                    if verts.len() >= 2 {
                        let pts: Vec<Vec3> = verts.iter()
                            .map(|v| Vec3::new(v.location.x as f32, v.location.z as f32, v.location.y as f32))
                            .collect();
                        draw_data.polylines.push((pts, pl.is_closed(), poly_color));
                    }
                }
                _ => {}
            }
        }
        println!("DXF 加载完成: {:?}, 线段: {}, 圆: {}, 弧: {}, 多段线: {}", 
            event.path, draw_data.lines.len(), draw_data.circles.len(), 
            draw_data.arcs.len(), draw_data.polylines.len());
    }
}

/// DXF Gizmos 绘制系统 - 每帧绘制
pub fn dxf_gizmos_system(
    mut gizmos: Gizmos,
    draw_data: Res<DxfDrawData>,
) {
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
        draw_arc(&mut gizmos, *center, *radius, *start_angle, *end_angle, *color);
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
fn draw_arc(gizmos: &mut Gizmos, center: Vec3, radius: f32, start_angle: f32, end_angle: f32, color: Color) {
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
