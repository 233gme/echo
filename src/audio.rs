use anyhow::Result;

/// Заглушка для аудио записи
pub struct AudioRecorder {
    enabled: bool,
}

impl AudioRecorder {
    pub fn new(enabled: bool) -> Self {
        Self { enabled }
    }

    pub fn start(&self) -> Result<()> {
        if self.enabled {
            println!("Audio recording started");
        }
        Ok(())
    }

    pub fn stop(&self) -> Result<()> {
        if self.enabled {
            println!("Audio recording stopped");
        }
        Ok(())
    }
}

impl Default for AudioRecorder {
    fn default() -> Self {
        Self::new(true)
    }
}
