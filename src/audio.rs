//! Audio recording facade — delegates to the ScreenCaptureKit capture worker.

use std::path::PathBuf;

use crate::capture;

/// Result of a finished capture (finalized file path on success).
pub type CaptureResult = capture::CaptureResult;

/// Start recording system audio to `file_path`.
///
/// Returns immediately — capture runs on a dedicated worker thread.
pub async fn start_recording(file_path: String) {
    log::info!("Starting recording to: {}", file_path);
    capture::start_audio_capture(file_path);
}

/// Stop the active recording. Finalization (WAV → M4A) continues on the
/// worker thread; poll [`try_take_completion`] to know when the file is ready.
pub fn stop_recording() {
    capture::stop_audio_capture();
    log::info!("Recording stop requested");
}

pub fn is_recording() -> bool {
    capture::is_capturing()
}

/// Non-blocking poll for the capture's final result.
///
/// Returns `Some(Ok(path))` once the audio file is finalized, `Some(Err(_))`
/// if capture failed, or `None` while still finalizing / idle. Each finished
/// capture yields exactly one result.
pub fn try_take_completion() -> Option<Result<PathBuf, String>> {
    capture::try_take_completion()
}
