//! Audio recording facade — delegates to ScreenCaptureKit capture.

use std::sync::atomic::{AtomicBool, Ordering};

static RECORDING: AtomicBool = AtomicBool::new(false);

pub async fn start_recording(file_path: String) {
    RECORDING.store(true, Ordering::SeqCst);
    log::info!("Starting recording to: {}", file_path);
    crate::capture::start_audio_capture(file_path);
}

pub fn stop_recording() {
    RECORDING.store(false, Ordering::SeqCst);
    crate::capture::stop_audio_capture();
    log::info!("Recording stopped");
}

pub fn is_recording() -> bool {
    RECORDING.load(Ordering::SeqCst)
}

