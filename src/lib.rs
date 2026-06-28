pub mod config;
pub mod audio;
pub mod capture;

pub use config::{Config, AudioConfig, CaptureConfig, get_config_path, load_config, save_config};
pub use audio::AudioRecorder;
pub use capture::ScreenCapture;

/// Основной API приложения
pub struct EchoApp {
    config: Config,
    audio_recorder: AudioRecorder,
    screen_capture: ScreenCapture,
}

impl EchoApp {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            audio_recorder: AudioRecorder::new(config.audio.enabled),
            screen_capture: ScreenCapture::new(config.capture.enabled),
        }
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    pub fn start_capture(&self) -> anyhow::Result<()> {
        self.audio_recorder.start()?;
        self.screen_capture.start()?;
        Ok(())
    }

    pub fn stop_capture(&self) -> anyhow::Result<()> {
        self.audio_recorder.stop()?;
        self.screen_capture.stop()?;
        Ok(())
    }
}

impl Default for EchoApp {
    fn default() -> Self {
        Self::new(Config::default())
    }
}
