use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub recordings_dir: PathBuf,
    pub temp_dir: PathBuf,
    pub cache_dir: PathBuf,
    pub logs_dir: PathBuf,
    pub obsidian_vault: PathBuf,
    pub config_path: PathBuf,
    pub voice_reference: PathBuf,
    pub db_path: PathBuf,
    pub backend_url: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        let base = dirs::home_dir().unwrap().join(".meeting_assistant");

        Self {
            recordings_dir: base.join("recordings"),
            temp_dir: base.join("temp"),
            cache_dir: base.join("cache"),
            logs_dir: base.join("logs"),
            obsidian_vault: dirs::home_dir().unwrap().join("Obsidian").join("Meetings"),
            config_path: base.join("config.yaml"),
            voice_reference: base.join("voice_reference.wav"),
            db_path: base.join("db.sqlite"),
            backend_url: "http://127.0.0.1:8000".to_string(),
        }
    }
}

impl AppConfig {
    pub fn load() -> Self {
        let config_path = dirs::home_dir()
            .unwrap()
            .join(".meeting_assistant")
            .join("config.yaml");

        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path).unwrap();
            serde_yaml::from_str(&content).unwrap_or_default()
        } else {
            let config = Self::default();
            config.save();
            config
        }
    }

    pub fn save(&self) {
        let content = serde_yaml::to_string(self).unwrap();
        std::fs::create_dir_all(self.config_path.parent().unwrap()).unwrap();
        std::fs::write(&self.config_path, content).unwrap();
    }

    pub fn ensure_dirs(&self) {
        for dir in [
            &self.recordings_dir,
            &self.temp_dir,
            &self.cache_dir,
            &self.logs_dir,
        ] {
            std::fs::create_dir_all(dir).unwrap();
        }
    }

    pub fn get_recording_path(&self) -> String {
        let now = chrono::Local::now();
        let filename = format!("meeting_{}.m4a", now.format("%Y-%m-%d_%H%M%S"));
        self.recordings_dir
            .join(filename)
            .to_string_lossy()
            .to_string()
    }
}
