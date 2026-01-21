use crate::state::AppState;

use super::{FileTree, LoadDxfMessage, Project};
use bevy::{
    prelude::*,
    window::{PrimaryWindow, Window},
};
use bevy_egui::*;
use egui::*;

pub fn in_project_ui_system(
    mut contexts: EguiContexts,
    _window: Single<&mut Window, With<PrimaryWindow>>,
    project: Res<Project>,
    mut next_state: ResMut<NextState<AppState>>,
    mut file_tree: Local<Option<FileTree>>,
    mut dxf_messages: MessageWriter<LoadDxfMessage>,
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

    // 初始化文件树（只在第一次运行时）或路径不一致时重新创建
    let should_recreate_tree = match file_tree.as_ref() {
        None => true,
        Some(tree) => !tree.is_same_root_path(&project.path),
    };
    if should_recreate_tree {
        *file_tree = Some(FileTree::new(project.path.clone()));
    }

    // 左侧面板
    SidePanel::left("left_panel")
        .resizable(true)
        .min_width(100.0)
        .default_width(200.0)
        .show(ctx, |ui| {
            ui.heading("文件树");
            ui.separator();
            if let Some(file_tree) = file_tree.as_mut() {
                file_tree.show(ui);
            }
        });

    // 显示对话框和处理事件
    if let Some(file_tree) = file_tree.as_mut() {
        file_tree.show_new_item_dialog(ctx);
        file_tree.show_dxf_json_viewer(ctx);
        file_tree.process_pending_events(&mut dxf_messages);
    }

    Ok(())
}
