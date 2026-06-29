//! ScreenCaptureKit audio capture for macOS 13.0+.
//!
//! # Ownership model
//!
//! A single dedicated worker thread owns the `SCStream` for its entire
//! lifecycle (construct → `start_capture` → wait for stop signal →
//! `stop_capture` → finalize file). This is critical because in
//! `screencapturekit` 8.0.0 `start_capture()` does **not** block until
//! `stop_capture()` — it returns as soon as capture has *started*. If the
//! stream is dropped right after `start_capture()` returns (as the previous
//! implementation did by `take()`-ing it in a throwaway thread), the capture
//! is torn down immediately and `stop_capture()` later operates on a dead
//! stream. Keeping the stream alive on one worker thread is what makes
//! "Start" → responsive UI → "Stop" → saved file actually work.
//!
//! `SCStream` is `Send + Sync` in 8.0.0, so the worker thread can own it
//! directly without any `unsafe` wrapping.

use std::fs::File;
use std::io::{Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Mutex};
use std::time::Duration;

use screencapturekit::prelude::*;

/// Result of a capture run: the finalized audio file path on success,
/// or an error message on failure.
pub type CaptureResult = Result<PathBuf, String>;

/// Worker → main completion channel. `try_take_completion` drains exactly one
/// result (the one from the active capture).
static COMPLETION_RX: Mutex<Option<mpsc::Receiver<CaptureResult>>> = Mutex::new(None);

/// Main → worker stop signal.
static STOP_TX: Mutex<Option<mpsc::Sender<()>>> = Mutex::new(None);

static CAPTURING: AtomicBool = AtomicBool::new(false);

/// Start capturing system audio to the given file path.
///
/// `file_path` is the desired final path (e.g. `meeting_*.m4a`); the WAV
/// intermediate is written alongside it with a `.wav` extension and removed
/// after conversion.
pub fn start_audio_capture(file_path: String) {
    if CAPTURING.load(Ordering::SeqCst) {
        log::warn!("Audio capture already running");
        return;
    }

    CAPTURING.store(true, Ordering::SeqCst);
    log::info!("Starting audio capture to: {}", file_path);

    let (stop_tx, stop_rx) = mpsc::channel::<()>();
    let (completion_tx, completion_rx) = mpsc::channel::<CaptureResult>();

    *STOP_TX.lock().unwrap() = Some(stop_tx);
    *COMPLETION_RX.lock().unwrap() = Some(completion_rx);

    std::thread::spawn(move || {
        let result = run_capture(&file_path, stop_rx);
        CAPTURING.store(false, Ordering::SeqCst);
        *STOP_TX.lock().unwrap() = None;
        let _ = completion_tx.send(result);
        log::info!("Audio capture thread exited");
    });
}

/// Stop the current audio capture. Safe to call when not capturing.
pub fn stop_audio_capture() {
    if !CAPTURING.load(Ordering::SeqCst) {
        return;
    }
    log::info!("Stop audio capture requested");
    // Tell the handler to stop appending samples immediately, then signal the
    // worker to call stop_capture().
    CAPTURING.store(false, Ordering::SeqCst);
    if let Ok(guard) = STOP_TX.lock() {
        if let Some(sender) = guard.as_ref() {
            let _ = sender.send(());
        }
    }
}

/// Check whether capture is currently active.
pub fn is_capturing() -> bool {
    CAPTURING.load(Ordering::SeqCst)
}

/// Non-blocking poll for the result of a finished capture.
///
/// Returns `Some(result)` exactly once after the worker has finalized (or
/// failed), consuming the completion receiver so subsequent calls return
/// `None`. The caller (main loop) uses this to know when the recording file
/// is ready to be sent to the backend.
pub fn try_take_completion() -> Option<CaptureResult> {
    let rx = COMPLETION_RX.lock().unwrap().take()?;
    match rx.try_recv() {
        Ok(result) => Some(result),
        Err(mpsc::TryRecvError::Empty) => {
            // Not ready yet — put the receiver back for the next poll.
            *COMPLETION_RX.lock().unwrap() = Some(rx);
            None
        }
        Err(mpsc::TryRecvError::Disconnected) => {
            // Worker died without sending; treat as error.
            Some(Err("Capture worker terminated unexpectedly".to_string()))
        }
    }
}

/// Worker entry point. Owns the `SCStream` for its whole lifetime.
fn run_capture(file_path: &str, stop_rx: mpsc::Receiver<()>) -> CaptureResult {
    inner_run_capture(file_path, stop_rx).map_err(|e| {
        log::error!("Audio capture error: {}", e);
        e.to_string()
    })
}

fn inner_run_capture(
    file_path: &str,
    stop_rx: mpsc::Receiver<()>,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let content = SCShareableContent::get()?;
    let display = content
        .displays()
        .into_iter()
        .next()
        .ok_or("No display found")?;

    let filter = SCContentFilter::create()
        .with_display(&display)
        .with_excluding_windows(&[])
        .build();

    let config = SCStreamConfiguration::new()
        .with_width(1)
        .with_height(1)
        .with_pixel_format(PixelFormat::BGRA)
        .with_captures_audio(true)
        .with_sample_rate(48000)
        .with_channel_count(2);

    let path = Path::new(file_path);
    let wav_path = path.with_extension("wav");
    let wav_writer = std::sync::Arc::new(std::sync::Mutex::new(WavWriter::new(
        &wav_path, 48000, 2,
    )?));
    let wav_writer_handler = wav_writer.clone();

    // SCStream is owned by this worker thread for the entire capture. It must
    // NOT be dropped until after stop_capture().
    let mut stream = SCStream::new(&filter, &config);

    stream.add_output_handler(
        move |sample: CMSampleBuffer, of_type: SCStreamOutputType| {
            // Stop appending as soon as a stop is requested.
            if !CAPTURING.load(Ordering::SeqCst) {
                return;
            }
            if of_type != SCStreamOutputType::Audio {
                return;
            }
            if let Some(buffer_list) = sample.audio_buffer_list() {
                for i in 0..buffer_list.num_buffers() {
                    if let Some(buffer) = buffer_list.get(i) {
                        let data = buffer.data();
                        if let Ok(mut writer) = wav_writer_handler.lock() {
                            writer.write_raw(data);
                        }
                    }
                }
            } else if let Some(data_buffer) = sample.data_buffer() {
                let length = data_buffer.data_length();
                if length > 0 {
                    if let Some(data) = data_buffer.copy_data_bytes(0, length) {
                        if let Ok(mut writer) = wav_writer_handler.lock() {
                            writer.write_raw(&data);
                        }
                    }
                }
            }
        },
        SCStreamOutputType::Audio,
    );

    // start_capture() returns once capture has begun (it is NOT a blocking
    // call that runs until stop). The stream stays alive on this thread.
    log::info!("Starting stream...");
    stream.start_capture()?;
    log::info!("Stream started, recording in progress");

    // Wait for the stop signal from the main thread.
    let _ = stop_rx.recv();
    log::info!("Stop signal received, stopping stream...");

    // Now that we're stopping, make sure the handler stops appending even
    // before the stream is torn down.
    CAPTURING.store(false, Ordering::SeqCst);

    match stream.stop_capture() {
        Ok(()) => log::info!("Stream stopped"),
        Err(e) => log::warn!("stop_capture returned error (ignoring): {}", e),
    }

    // Drop the stream first so the output handler (which holds the other
    // WavWriter Arc clone) is released before we finalize.
    drop(stream);

    // Give any in-flight samples a moment to flush, then finalize the WAV.
    std::thread::sleep(Duration::from_millis(200));

    // At this point the handler's Arc clone is dropped (stream gone), so we
    // can unwrap our clone and finalize. Fall back to locking if, for any
    // reason, an extra reference lingers.
    match std::sync::Arc::try_unwrap(wav_writer) {
        Ok(writer) => writer.into_inner()?.finalize()?,
        Err(arc) => arc.lock()?.finalize()?,
    }
    log::info!("WAV saved: {}", wav_path.display());

    // Convert WAV → M4A (AAC) via the bundled `afconvert` tool.
    let m4a_path = path.with_extension("m4a");
    let status = std::process::Command::new("afconvert")
        .args([
            "-f",
            "m4af",
            "-d",
            "aac",
            "-b",
            "128000",
            wav_path.to_str().unwrap(),
            m4a_path.to_str().unwrap(),
        ])
        .status()?;

    if status.success() {
        let _ = std::fs::remove_file(&wav_path);
        log::info!("M4A saved: {}", m4a_path.display());
        Ok(m4a_path)
    } else {
        log::warn!(
            "afconvert failed, keeping WAV: {}",
            wav_path.display()
        );
        Ok(wav_path)
    }
}

/// Streaming WAV writer (float32, little-endian).
struct WavWriter {
    file: File,
    data_size: u64,
}

impl WavWriter {
    fn new(path: &Path, sample_rate: u32, channels: u16) -> Result<Self, std::io::Error> {
        let mut file = File::create(path)?;
        file.write_all(b"RIFF")?;
        file.write_all(&0u32.to_le_bytes())?;
        file.write_all(b"WAVE")?;
        file.write_all(b"fmt ")?;
        file.write_all(&16u32.to_le_bytes())?;
        file.write_all(&3u16.to_le_bytes())?; // IEEE float
        file.write_all(&channels.to_le_bytes())?;
        file.write_all(&sample_rate.to_le_bytes())?;
        let byte_rate = sample_rate * channels as u32 * 4;
        file.write_all(&byte_rate.to_le_bytes())?;
        file.write_all(&(channels * 4).to_le_bytes())?;
        file.write_all(&32u16.to_le_bytes())?;
        file.write_all(b"data")?;
        file.write_all(&0u32.to_le_bytes())?;
        Ok(WavWriter {
            file,
            data_size: 0,
        })
    }

    fn write_raw(&mut self, data: &[u8]) {
        if let Err(e) = self.file.write_all(data) {
            log::error!("WAV write error: {}", e);
        } else {
            self.data_size += data.len() as u64;
        }
    }

    /// Patch the RIFF/data size headers now that we know the final length.
    fn finalize(&mut self) -> Result<(), std::io::Error> {
        self.file.seek(SeekFrom::Start(4))?;
        let file_size = (self.data_size + 36) as u32;
        self.file.write_all(&file_size.to_le_bytes())?;
        self.file.seek(SeekFrom::Start(40))?;
        self.file.write_all(&(self.data_size as u32).to_le_bytes())?;
        self.file.flush()?;
        Ok(())
    }
}
