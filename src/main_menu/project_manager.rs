use serde::{Deserialize, Serialize};
use std::path::PathBuf;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectInfo {
    pub name: String,
    pub path: PathBuf,
    pub last_opened: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProjectManager {
    pub recent_projects: Vec<ProjectInfo>,
}

impl ProjectManager {
    const CONFIG_FILE: &'static str = "open_cad_projects.json";

    pub fn load() -> Self {
        if let Some(config_dir) = dirs::config_dir() {
            let config_path = config_dir.join(Self::CONFIG_FILE);
            if let Ok(content) = std::fs::read_to_string(&config_path) {
                if let Ok(manager) = serde_json::from_str(&content) {
                    return manager;
                }
            }
        }
        Self::default()
    }

    pub fn save(&self) {
        if let Some(config_dir) = dirs::config_dir() {
            let config_path = config_dir.join(Self::CONFIG_FILE);
            if let Ok(content) = serde_json::to_string_pretty(self) {
                let _ = std::fs::write(&config_path, content);
            }
        }
    }

    pub fn add_project(&mut self, path: PathBuf) {
        // 移除已存在的同路径项目
        self.recent_projects.retain(|p| p.path != path);

        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Unknown")
            .to_string();

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        self.recent_projects.insert(
            0,
            ProjectInfo {
                name,
                path,
                last_opened: timestamp,
            },
        );

        // 保留最近 10 个项目
        self.recent_projects.truncate(10);
    }

    pub fn remove_project(&mut self, index: usize) {
        if index < self.recent_projects.len() {
            self.recent_projects.remove(index);
        }
    }
}
