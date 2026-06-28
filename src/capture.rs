use anyhow::Result;

/// Заглушка для ScreenCaptureKit
pub struct ScreenCapture {
    enabled: bool,
}

impl ScreenCapture {
    pub fn new(enabled: bool) -> Self {
        Self { enabled }
    }

    pub fn start(&self) -> Result<()> {
        if self.enabled {
            println!("Screen capture started");
        }
        Ok(())
    }

    pub fn stop(&self) -> Result<()> {
        if self.enabled {
            println!("Screen capture stopped");
        }
        Ok(())
    }
}

impl Default for ScreenCapture {
    fn default() -> Self {
        Self::new(true)
    }
}
