use super::project_manager::ProjectManager;
use crate::in_project::Project;
use crate::state::AppState;
use bevy::prelude::*;
use bevy_egui::*;
use egui::*;
use std::path::PathBuf;
use std::sync::Mutex;
use std::thread;

static DIALOG_RESULT: Mutex<Option<Option<std::path::PathBuf>>> = Mutex::new(None);

#[derive(Resource)]
pub struct ProjectListState {
    pub manager: ProjectManager,
    pub is_dialog_open: bool,
}

impl Default for ProjectListState {
    fn default() -> Self {
        Self {
            manager: ProjectManager::load(),
            is_dialog_open: false,
        }
    }
}

pub fn main_menu_ui_system(
    mut contexts: EguiContexts,
    mut state: ResMut<ProjectListState>,
    mut commands: Commands,
    mut next_state: ResMut<NextState<AppState>>,
) {
    let ctx = contexts.ctx_mut().unwrap();

    CentralPanel::default().show(ctx, |ui| {
        ui.heading("项目管理");
        ui.add_space(20.0);

        if ui.button("📁 导入项目").clicked() && !state.is_dialog_open {
            state.is_dialog_open = true;
            open_folder_dialog_in_thread();
        }

        if ui.button("📁 打开空项目").clicked() {
            commands.insert_resource(Project::new(PathBuf::from("空项目")));
            next_state.set(AppState::InPreject);
        }

        // 检查对话框结果
        if state.is_dialog_open {
            if let Ok(result_guard) = DIALOG_RESULT.lock() {
                if let Some(result) = result_guard.as_ref() {
                    state.is_dialog_open = false;
                    if let Some(path) = result {
                        state.manager.add_project(path.clone());
                        state.manager.save();
                        println!("项目已导入: {:?}", path);
                    }
                    drop(result_guard);
                    if let Ok(mut guard) = DIALOG_RESULT.lock() {
                        *guard = None;
                    }
                }
            }
        }

        ui.add_space(20.0);
        ui.separator();
        ui.add_space(10.0);
        ui.heading("最近项目");
        ui.add_space(10.0);

        if state.manager.recent_projects.is_empty() {
            ui.label("暂无最近项目");
        } else {
            let mut projects_to_remove = Vec::new();
            let mut project_to_open = None;

            for (index, project) in state.manager.recent_projects.iter().enumerate() {
                ui.horizontal(|ui| {
                    ui.label("📂");
                    ui.label(&project.name);

                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ui.button("❌").clicked() {
                            projects_to_remove.push(index);
                        }
                        if ui.button("📂").clicked() {
                            project_to_open = Some(project.clone());
                        }
                    });
                });

                ui.label(format!("路径: {}", project.path.display()));
                ui.label(format!(
                    "最后打开: {}",
                    format_timestamp(project.last_opened)
                ));
                ui.add_space(10.0);
            }

            // 移除项目
            for &index in projects_to_remove.iter().rev() {
                state.manager.remove_project(index);
                state.manager.save();
            }

            // TODO 打开项目
            if let Some(project) = project_to_open {
                commands.insert_resource(Project::new(project.path));
                next_state.set(AppState::InPreject);
            }
        }
    });
}

fn format_timestamp(timestamp: u64) -> String {
    use chrono::{DateTime, Local};
    let datetime = DateTime::from_timestamp(timestamp as i64, 0)
        .unwrap_or_else(|| DateTime::from_timestamp(0, 0).unwrap())
        .with_timezone(&Local);
    datetime.format("%Y-%m-%d %H:%M").to_string()
}

#[cfg(not(target_arch = "wasm32"))]
fn open_folder_dialog_in_thread() {
    thread::spawn(move || {
        let home_dir = std::env::home_dir().unwrap();
        let result = rfd::FileDialog::new()
            .set_title("选择项目文件夹")
            .set_directory(&home_dir)
            .pick_folder();

        if let Ok(mut dialog_result) = DIALOG_RESULT.lock() {
            *dialog_result = Some(result);
        }
    });
}

#[cfg(target_arch = "wasm32")]
fn open_folder_dialog_in_thread() -> Option<String> {
    // 这里返回浏览器选中的文件 URL / 内容
    todo!("use <input type='file'> via web-sys")
}
