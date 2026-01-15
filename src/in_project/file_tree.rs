use bevy_egui::egui;
use dxf::Drawing;
use egui_ltreeview::TreeView;
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;

/// 文件树节点
#[derive(Debug, Clone)]
pub struct FileTreeNode {
    pub path: PathBuf,
    pub name: String,
    pub is_directory: bool,
    pub children: Vec<u64>,
    pub children_loaded: bool,
}

impl FileTreeNode {
    pub fn new(path: PathBuf, is_directory: bool) -> Self {
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("未知")
            .to_string();

        Self {
            path,
            name,
            is_directory,
            children: Vec::new(),
            children_loaded: false,
        }
    }
}

/// 新建对话框状态
#[derive(Default)]
pub struct NewItemDialog {
    pub open: bool,
    pub is_folder: bool,
    pub name: String,
    pub parent_path: PathBuf,
}

/// DXF查看器窗口状态
#[derive(Default)]
pub struct DxfViewer {
    pub open: bool,
    pub file_path: PathBuf,
    pub json_content: String,
    pub error: Option<String>,
}

/// 文件树组件
pub struct FileTree {
    root_path: PathBuf,
    nodes: HashMap<u64, FileTreeNode>,
    next_id: u64,
    /// 新建文件/文件夹对话框
    pub new_item_dialog: NewItemDialog,
    /// DXF查看器
    pub dxf_viewer: DxfViewer,
    /// 需要刷新的目录ID
    refresh_dir: Option<u64>,
}

impl FileTree {
    pub fn new(root_path: PathBuf) -> Self {
        let mut tree = Self {
            root_path: root_path.clone(),
            nodes: HashMap::new(),
            next_id: 1,
            new_item_dialog: NewItemDialog::default(),
            dxf_viewer: DxfViewer::default(),
            refresh_dir: None,
        };

        // 添加根节点
        let root_node = FileTreeNode::new(root_path, true);
        tree.nodes.insert(0, root_node);
        // 预加载根目录
        tree.load_children(0);

        tree
    }

    /// 加载目录子项（强制刷新）
    fn load_children_force(&mut self, parent_id: u64) {
        let parent_path = {
            if let Some(node) = self.nodes.get(&parent_id) {
                if !node.is_directory {
                    return;
                }
                node.path.clone()
            } else {
                return;
            }
        };

        // 先收集旧的子节点ID
        let old_children = if let Some(node) = self.nodes.get_mut(&parent_id) {
            let children = std::mem::take(&mut node.children);
            node.children_loaded = true;
            children
        } else {
            return;
        };

        // 递归删除旧子节点
        for child_id in old_children {
            self.remove_node_recursive(child_id);
        }

        let mut children = Vec::new();
        if let Ok(entries) = fs::read_dir(&parent_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                // 跳过隐藏文件
                if path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .map(|n| n.starts_with('.'))
                    .unwrap_or(false)
                {
                    continue;
                }
                let is_directory = path.is_dir();
                children.push(FileTreeNode::new(path, is_directory));
            }
        }

        // 排序：目录在前，按名称排序
        children.sort_by(|a, b| match (a.is_directory, b.is_directory) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
        });

        let mut child_ids = Vec::new();
        for child in children {
            let child_id = self.next_id;
            self.next_id += 1;
            self.nodes.insert(child_id, child);
            child_ids.push(child_id);
        }

        if let Some(node) = self.nodes.get_mut(&parent_id) {
            node.children = child_ids;
        }
    }

    /// 懒加载目录子项（仅首次）
    fn load_children(&mut self, parent_id: u64) {
        if let Some(node) = self.nodes.get(&parent_id) {
            if node.children_loaded || !node.is_directory {
                return;
            }
        }
        self.load_children_force(parent_id);
    }

    /// 递归移除节点
    fn remove_node_recursive(&mut self, node_id: u64) {
        if let Some(node) = self.nodes.remove(&node_id) {
            for child_id in node.children {
                self.remove_node_recursive(child_id);
            }
        }
    }

    /// 查找路径对应的节点ID
    fn find_node_id_by_path(&self, path: &PathBuf) -> Option<u64> {
        for (id, node) in &self.nodes {
            if &node.path == path {
                return Some(*id);
            }
        }
        None
    }

    /// 显示文件树
    pub fn show(&mut self, ui: &mut egui::Ui) {
        // 处理待刷新的目录
        if let Some(dir_id) = self.refresh_dir.take() {
            self.load_children_force(dir_id);
        }

        let tree_id = ui.make_persistent_id("file_tree");

        // 收集节点信息快照
        let nodes_snapshot: HashMap<u64, (PathBuf, bool, String)> = self
            .nodes
            .iter()
            .map(|(id, node)| (*id, (node.path.clone(), node.is_directory, node.name.clone())))
            .collect();

        let mut context_action: Option<ContextAction> = None;
        let mut dirs_to_refresh: Vec<u64> = Vec::new();

        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Truncate);

                let (response, actions) = TreeView::<u64>::new(tree_id)
                    .allow_drag_and_drop(false)
                    .fallback_context_menu(|ui, selected_nodes| {
                        // 设置菜单最小宽度
                        ui.set_min_width(150.0);

                        // 获取第一个选中节点的信息
                        if let Some(&node_id) = selected_nodes.first() {
                            if let Some((path, is_dir, _name)) = nodes_snapshot.get(&node_id) {
                                // 打开按钮 - 所有文件/文件夹都有
                                if ui.button("📂 打开").clicked() {
                                    context_action = Some(ContextAction::Open(path.clone()));
                                    ui.close();
                                }

                                ui.separator();

                                let target_path = if *is_dir {
                                    path.clone()
                                } else {
                                    path.parent()
                                        .map(|p| p.to_path_buf())
                                        .unwrap_or_else(|| path.clone())
                                };

                                if ui.button("📁 新建文件夹").clicked() {
                                    context_action =
                                        Some(ContextAction::NewFolder(target_path.clone()));
                                    ui.close();
                                }

                                if ui.button("� 新建文件").clicked() {
                                    context_action =
                                        Some(ContextAction::NewFile(target_path.clone()));
                                    ui.close();
                                }

                                ui.separator();

                                if ui.button("🔄 刷新").clicked() {
                                    context_action = Some(ContextAction::Refresh(target_path));
                                    ui.close();
                                }
                            }
                        }
                    })
                    .show(ui, |builder| {
                        self.build_tree(builder, 0);
                    });

                // 处理动作
                for action in actions {
                    match action {
                        // 处理选中变化时展开目录
                        egui_ltreeview::Action::SetSelected(selected) => {
                            for node_id in selected {
                                if let Some((_, is_dir, _)) = nodes_snapshot.get(&node_id) {
                                    if *is_dir {
                                        dirs_to_refresh.push(node_id);
                                    }
                                }
                            }
                        }
                        // 双击激活
                        egui_ltreeview::Action::Activate(activate) => {
                            for node_id in activate.selected {
                                if let Some((path, is_dir, _)) = nodes_snapshot.get(&node_id) {
                                    if *is_dir {
                                        dirs_to_refresh.push(node_id);
                                    } else {
                                        // 双击文件时打开
                                        context_action = Some(ContextAction::Open(path.clone()));
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
                let _ = response;
            });

        // 刷新展开的目录
        for dir_id in dirs_to_refresh {
            self.load_children_force(dir_id);
        }

        // 处理右键菜单动作
        if let Some(action) = context_action {
            match action {
                ContextAction::Open(path) => {
                    self.handle_open(&path);
                }
                ContextAction::NewFolder(path) => {
                    self.new_item_dialog = NewItemDialog {
                        open: true,
                        is_folder: true,
                        name: String::new(),
                        parent_path: path,
                    };
                }
                ContextAction::NewFile(path) => {
                    self.new_item_dialog = NewItemDialog {
                        open: true,
                        is_folder: false,
                        name: String::new(),
                        parent_path: path,
                    };
                }
                ContextAction::Refresh(path) => {
                    if let Some(dir_id) = self.find_node_id_by_path(&path) {
                        self.refresh_dir = Some(dir_id);
                    }
                }
            }
        }
    }

    /// 处理打开操作
    fn handle_open(&mut self, path: &PathBuf) {
        if path.is_dir() {
            // 目录：刷新
            if let Some(dir_id) = self.find_node_id_by_path(path) {
                self.refresh_dir = Some(dir_id);
            }
        } else {
            // 文件：根据扩展名处理
            let ext = path
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| e.to_lowercase())
                .unwrap_or_default();

            match ext.as_str() {
                "dxf" => self.open_dxf_file(path),
                _ => {
                    // 其他文件暂不处理
                }
            }
        }
    }

    /// 构建树节点
    fn build_tree(&self, builder: &mut egui_ltreeview::TreeViewBuilder<u64>, node_id: u64) {
        if let Some(node) = self.nodes.get(&node_id) {
            if node.is_directory {
                let display_name = format!("� {}", node.name);
                builder.dir(node_id, &display_name);

                for &child_id in &node.children {
                    self.build_tree(builder, child_id);
                }

                builder.close_dir();
            } else {
                let icon = self.get_file_icon(&node.name);
                let display_name = format!("{} {}", icon, node.name);
                builder.leaf(node_id, &display_name);
            }
        }
    }

    /// 根据文件扩展名获取图标
    fn get_file_icon(&self, name: &str) -> &'static str {
        let ext = name.rsplit('.').next().unwrap_or("").to_lowercase();
        match ext.as_str() {
            "dxf" => "📐",
            "dwg" => "📐",
            "rs" => "🦀",
            "toml" => "⚙️",
            "json" => "📋",
            "txt" => "📝",
            "md" => "📖",
            _ => "📄",
        }
    }

    /// 显示新建对话框
    pub fn show_new_item_dialog(&mut self, ctx: &egui::Context) {
        if !self.new_item_dialog.open {
            return;
        }

        let title = if self.new_item_dialog.is_folder {
            "新建文件夹"
        } else {
            "新建文件"
        };

        let mut should_close = false;
        let mut should_create = false;

        egui::Window::new(title)
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("名称:");
                    let response = ui.text_edit_singleline(&mut self.new_item_dialog.name);
                    if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        should_create = true;
                    }
                });

                ui.add_space(10.0);

                ui.horizontal(|ui| {
                    if ui.button("创建").clicked() {
                        should_create = true;
                    }
                    if ui.button("取消").clicked() {
                        should_close = true;
                    }
                });
            });

        if should_create && !self.new_item_dialog.name.is_empty() {
            let new_path = self
                .new_item_dialog
                .parent_path
                .join(&self.new_item_dialog.name);

            let result = if self.new_item_dialog.is_folder {
                fs::create_dir(&new_path)
            } else {
                File::create(&new_path).map(|_| ())
            };

            if result.is_ok() {
                if let Some(parent_id) =
                    self.find_node_id_by_path(&self.new_item_dialog.parent_path)
                {
                    self.refresh_dir = Some(parent_id);
                }
            }

            should_close = true;
        }

        if should_close {
            self.new_item_dialog.open = false;
            self.new_item_dialog.name.clear();
        }
    }

    /// 打开DXF文件
    fn open_dxf_file(&mut self, path: &PathBuf) {
        self.dxf_viewer.file_path = path.clone();
        self.dxf_viewer.error = None;

        match Drawing::load_file(path) {
            Ok(drawing) => match serde_json::to_string_pretty(&drawing) {
                Ok(json) => {
                    self.dxf_viewer.json_content = json;
                    self.dxf_viewer.open = true;
                }
                Err(e) => {
                    self.dxf_viewer.error = Some(format!("JSON序列化失败: {}", e));
                    self.dxf_viewer.open = true;
                }
            },
            Err(e) => {
                self.dxf_viewer.error = Some(format!("DXF解析失败: {}", e));
                self.dxf_viewer.open = true;
            }
        }
    }

    /// 显示DXF查看器窗口
    pub fn show_dxf_viewer(&mut self, ctx: &egui::Context) {
        if !self.dxf_viewer.open {
            return;
        }

        let file_name = self
            .dxf_viewer
            .file_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("DXF");

        let title = format!("DXF查看器 - {}", file_name);

        egui::Window::new(title)
            .default_size([600.0, 400.0])
            .open(&mut self.dxf_viewer.open)
            .show(ctx, |ui| {
                if let Some(ref error) = self.dxf_viewer.error {
                    ui.colored_label(egui::Color32::RED, error);
                } else {
                    ui.horizontal(|ui| {
                        if ui.button("📋 复制").clicked() {
                            ctx.copy_text(self.dxf_viewer.json_content.clone());
                        }
                        if ui.button("💾 保存JSON").clicked() {
                            let json_path = self.dxf_viewer.file_path.with_extension("dxf.json");
                            if let Ok(mut file) = File::create(&json_path) {
                                let _ = file.write_all(self.dxf_viewer.json_content.as_bytes());
                            }
                        }
                    });

                    ui.separator();

                    egui::ScrollArea::both()
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            ui.add(
                                egui::TextEdit::multiline(&mut self.dxf_viewer.json_content)
                                    .font(egui::TextStyle::Monospace)
                                    .code_editor()
                                    .desired_width(f32::INFINITY),
                            );
                        });
                }
            });
    }

    /// 检查给定路径是否与当前根路径相同
    pub fn is_same_root_path(&self, path: &PathBuf) -> bool {
        &self.root_path == path
    }
}

/// 右键菜单动作
enum ContextAction {
    Open(PathBuf),
    NewFolder(PathBuf),
    NewFile(PathBuf),
    Refresh(PathBuf),
}
