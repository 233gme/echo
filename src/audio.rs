use std::sync::atomic::{AtomicBool, Ordering};

static RECORDING: AtomicBool = AtomicBool::new(false);

pub async fn start_recording(file_path: String) {
    RECORDING.store(true, Ordering::SeqCst);
    log::info!("Starting recording to: {}", file_path);
    // TODO: Implement ScreenCaptureKit audio capture
}

pub fn stop_recording() {
    RECORDING.store(false, Ordering::SeqCst);
    log::info!("Recording stopped");
}

pub fn is_recording() -> bool {
    RECORDING.load(Ordering::SeqCst)
}
