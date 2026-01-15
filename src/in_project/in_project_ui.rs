use crate::state::AppState;

use super::CurrentProject;
use bevy::{
    prelude::*,
    window::{PrimaryWindow, Window},
};
use bevy_egui::*;
use egui::*;
use egui_ltreeview::TreeView;

pub fn in_project_ui_system(
    mut contexts: EguiContexts,
    window: Single<&mut Window, With<PrimaryWindow>>,
    mut project: ResMut<CurrentProject>,
    mut next_state: ResMut<NextState<AppState>>,
) -> Result {
    let ctx = contexts.ctx_mut()?;

    // 顶部菜单栏
    TopBottomPanel::top("title_bar").show(ctx, |ui| {
        MenuBar::new().ui(ui, |ui: &mut Ui| {
            if ui.button("退出").clicked() {
                next_state.set(AppState::MainMenu);
            }
        });
    });
    
    // 左侧面板
    SidePanel::left("left_panel")
        .resizable(true)
        .min_width(100.0)
        .default_width(200.0)
        .show(ctx, |ui| {
            let id = ui.make_persistent_id("Names tree view");
            TreeView::new(id).show(ui, |builder| {
                builder.dir(0, "Root");
                builder.leaf(1, "Ava");
                builder.leaf(2, "Benjamin");
                builder.leaf(3, "Charlotte");
                builder.close_dir();
            });
        });

    // // // 右侧面板
    // // SidePanel::right("right_panel")
    // //     .resizable(true)
    // //     .min_width(100.0)
    // //     .default_width(200.0)
    // //     .show(ctx, |ui| {
    // //         ui.heading("右侧面板");
    // //     });

    // // 底部状态栏
    // TopBottomPanel::bottom("bottom_bar").show(ctx, |ui| {
    //     ui.label("底部状态栏");
    //     ui.allocate_rect(ui.available_rect_before_wrap(), egui::Sense::hover());
    // });

    Ok(())
}
