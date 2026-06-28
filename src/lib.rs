pub mod audio;
pub mod capture;
pub mod config;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeetingInfo {
    pub id: String,
    pub date: String,
    pub duration: String,
    pub speakers: Vec<String>,
    pub status: MeetingStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MeetingStatus {
    Recording,
    Processing { stage: String, progress: f32 },
    Completed,
    Error(String),
}
