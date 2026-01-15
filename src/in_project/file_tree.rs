use bevy_egui::egui;
use egui_ltreeview::TreeView;
use std::collections::HashMap;
use std::fs;
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

/// 文件树组件
pub struct FileTree {
    root_path: PathBuf,
    nodes: HashMap<u64, FileTreeNode>,
    next_id: u64,
}

impl FileTree {
    pub fn new(root_path: PathBuf) -> Self {
        let mut tree = Self {
            root_path: root_path.clone(),
            nodes: HashMap::new(),
            next_id: 1,
        };

        // 添加根节点
        let root_node = FileTreeNode::new(root_path, true);
        tree.nodes.insert(0, root_node);

        tree
    }

    /// 获取目录的子项
    fn load_children(&mut self, parent_id: u64) {
        if let Some(parent_node) = self.nodes.get_mut(&parent_id) {
            if parent_node.children_loaded || !parent_node.is_directory {
                return;
            }

            parent_node.children_loaded = true;
            let parent_path = parent_node.path.clone();

            let mut children = Vec::new();
            if let Ok(entries) = fs::read_dir(&parent_path) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    let is_directory = path.is_dir();
                    children.push(FileTreeNode::new(path, is_directory));
                }
            }

            // 按名称排序，目录在前
            children.sort_by(|a, b| match (a.is_directory, b.is_directory) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.name.cmp(&b.name),
            });

            let mut child_ids = Vec::new();
            for child in children {
                let child_id = self.next_id;
                self.next_id += 1;
                self.nodes.insert(child_id, child);
                child_ids.push(child_id);
            }

            // 更新父节点的子项列表
            if let Some(parent_node) = self.nodes.get_mut(&parent_id) {
                parent_node.children = child_ids;
            }
        }
    }

    /// 显示文件树
    pub fn show(&mut self, ui: &mut bevy_egui::egui::Ui) {
        let tree_id = ui.make_persistent_id("file_tree");

        // 预加载根目录的子项
        if let Some(root_node) = self.nodes.get(&0) {
            if !root_node.children_loaded {
                self.load_children(0);
            }
        }

        // 预加载所有已展开目录的子项
        let mut dirs_to_load = Vec::new();
        for (id, node) in &self.nodes {
            if node.is_directory && !node.children_loaded {
                dirs_to_load.push(*id);
            }
        }

        for dir_id in dirs_to_load {
            self.load_children(dir_id);
        }

        ui.allocate_ui_with_layout(
            egui::vec2(ui.available_width(), ui.spacing().interact_size.y),
            egui::Layout::left_to_right(egui::Align::Center),
            |ui| {
                ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Truncate);

                TreeView::new(tree_id)
                    .allow_drag_and_drop(false)
                    .show(ui, |builder| {
                        // 构建根目录
                        if let Some(root_node) = self.nodes.get(&0) {
                            builder.dir(0, &root_node.name);

                            // 构建子项
                            let child_ids = root_node.children.clone();
                            for child_id in child_ids {
                                Self::build_tree_node_static(&self.nodes, builder, child_id);
                            }

                            builder.close_dir();
                        }
                    });
            },
        );
    }

    /// 构建单个树节点（静态方法避免借用问题）
    fn build_tree_node_static(
        nodes: &HashMap<u64, FileTreeNode>,
        builder: &mut egui_ltreeview::TreeViewBuilder<u64>,
        child_id: u64,
    ) {
        if let Some(node) = nodes.get(&child_id) {
            if node.is_directory {
                // 为目录添加文件夹图标
                let display_name = format!("📁 {}", node.name);
                builder.dir(child_id, &display_name);

                // 构建子项
                let child_ids = node.children.clone();
                for sub_child_id in child_ids {
                    Self::build_tree_node_static(nodes, builder, sub_child_id);
                }

                builder.close_dir();
            } else {
                // 为文件添加文件图标
                let display_name = format!("📄 {}", node.name);
                builder.leaf(child_id, &display_name);
            }
        }
    }
    /// 获取节点路径
    pub fn get_node_path(&self, node_id: u64) -> Option<&PathBuf> {
        self.nodes.get(&node_id).map(|node| &node.path)
    }

    /// 获取文件树的根路径
    pub fn get_root_path(&self) -> &PathBuf {
        &self.root_path
    }

    /// 检查给定路径是否与当前根路径相同
    pub fn is_same_root_path(&self, path: &PathBuf) -> bool {
        &self.root_path == path
    }

    /// 获取所有节点
    pub fn get_nodes(&self) -> &HashMap<u64, FileTreeNode> {
        &self.nodes
    }
}
