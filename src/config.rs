use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub audio: AudioConfig,
    pub capture: CaptureConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AudioConfig {
    pub enabled: bool,
    pub sample_rate: u32,
    pub channels: u16,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CaptureConfig {
    pub enabled: bool,
    pub quality: u32,
    pub output_dir: PathBuf,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            audio: AudioConfig {
                enabled: true,
                sample_rate: 44100,
                channels: 2,
            },
            capture: CaptureConfig {
                enabled: true,
                quality: 90,
                output_dir: dirs::audio_dir()
                    .unwrap_or_else(|| PathBuf::from("."))
                    .join("echo"),
            },
        }
    }
}

pub fn get_config_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("echo")
        .join("config.yaml")
}

pub fn load_config() -> Result<Config> {
    let config_path = get_config_path();
    let content = std::fs::read_to_string(&config_path)?;
    let config: Config = serde_yaml::from_str(&content)?;
    Ok(config)
}

pub fn save_config(config: &Config) -> Result<()> {
    let config_path = get_config_path();
    std::fs::create_dir_all(config_path.parent().unwrap())?;
    let content = serde_yaml::to_string(config)?;
    std::fs::write(config_path, content)?;
    Ok(())
}
