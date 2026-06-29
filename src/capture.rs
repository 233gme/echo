//! ScreenCaptureKit audio capture for macOS 13.0+
//!
//! Captures system audio output and writes to WAV file.

use std::fs::File;
use std::io::{Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use screencapturekit::prelude::*;

static CAPTURING: AtomicBool = AtomicBool::new(false);

/// Start capturing system audio to the given file path.
/// Returns immediately; capture runs on a background thread.
pub fn start_audio_capture(file_path: String) {
    if CAPTURING.load(Ordering::SeqCst) {
        log::warn!("Audio capture already running");
        return;
    }

    CAPTURING.store(true, Ordering::SeqCst);
    log::info!("Starting audio capture to: {}", file_path);

    std::thread::spawn(move || {
        if let Err(e) = run_capture(&file_path) {
            log::error!("Audio capture error: {}", e);
        }
        CAPTURING.store(false, Ordering::SeqCst);
    });
}

/// Stop the current audio capture.
pub fn stop_audio_capture() {
    CAPTURING.store(false, Ordering::SeqCst);
    log::info!("Audio capture stop requested");
}

/// Check if capture is active.
pub fn is_capturing() -> bool {
    CAPTURING.load(Ordering::SeqCst)
}

fn run_capture(file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Get the primary display
    let content = SCShareableContent::get()?;
    let display = content
        .displays()
        .into_iter()
        .next()
        .ok_or("No display found")?;

    // Filter: capture the entire display (audio comes with it)
    let filter = SCContentFilter::create()
        .with_display(&display)
        .with_excluding_windows(&[])
        .build();

    // Config: audio only settings
    let config = SCStreamConfiguration::new()
        .with_width(1) // Minimal video — we discard it
        .with_height(1)
        .with_pixel_format(PixelFormat::BGRA)
        .with_captures_audio(true)
        .with_sample_rate(48000)
        .with_channel_count(2);

    // WAV file writer
    let path = Path::new(file_path);
    let wav_writer = Arc::new(Mutex::new(WavWriter::new(path, 48000, 2)?));
    let wav_writer_clone = wav_writer.clone();

    // Shared flag for the handler to check
    let capturing = Arc::new(AtomicBool::new(true));
    let capturing_handler = capturing.clone();

    // Start capture
    let mut stream = SCStream::new(&filter, &config);

    stream.add_output_handler(
        move |sample: CMSampleBuffer, of_type: SCStreamOutputType| {
            if !capturing_handler.load(Ordering::SeqCst) {
                return;
            }
            if of_type == SCStreamOutputType::Audio {
                if let Some(buffer_list) = sample.audio_buffer_list() {
                    // ScreenCaptureKit audio is typically float32
                    // Access the first buffer (stereo audio)
                    if let Some(buffer) = buffer_list.buffer(0) {
                        let data = buffer.data();
                        let mut writer = wav_writer_clone.lock().unwrap();
                        writer.write_raw(data);
                    }
                }
            }
        },
        SCStreamOutputType::Audio,
    );

    stream.start_capture()?;
    log::info!("Audio capture stream started");

    // Block until stop requested
    while CAPTURING.load(Ordering::SeqCst) {
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    stream.stop_capture()?;
    log::info!("Audio capture stream stopped");

    // Finalize WAV
    let wav_writer_final = wav_writer.clone();
    wav_writer_final.lock().unwrap().clone().finalize()?;
    let wav_path = wav_writer.lock().unwrap().file_path.clone();
    log::info!("WAV saved: {}", wav_path.display());

    // Convert to M4A using afconvert (macOS built-in)
    let m4a_path = path.with_extension("m4a");
    let status = std::process::Command::new("afconvert")
        .args(&[
            "-f",
            "m4af",
            "-d",
            "aac",
            wav_path.to_str().unwrap(),
            m4a_path.to_str().unwrap(),
        ])
        .status()?;

    if status.success() {
        std::fs::remove_file(&wav_path)?;
        log::info!("M4A saved: {}", m4a_path.display());
    } else {
        log::warn!("afconvert failed, keeping WAV: {}", wav_path.display());
    }

    Ok(())
}

/// Simple WAV file writer
#[derive(Clone)]
struct WavWriter {
    file_path: PathBuf,
    data_size: u64,
    sample_rate: u32,
    channels: u16,
}

impl WavWriter {
    fn new(path: &Path, sample_rate: u32, channels: u16) -> Result<Self, std::io::Error> {
        let file_path = path.with_extension("wav").to_path_buf();
        let mut file = File::create(&file_path)?;
        // Write placeholder header
        file.write_all(b"RIFF")?;
        file.write_all(&0u32.to_le_bytes())?; // file size - 8 (placeholder)
        file.write_all(b"WAVE")?;
        file.write_all(b"fmt ")?;
        file.write_all(&16u32.to_le_bytes())?; // fmt chunk size
        file.write_all(&3u16.to_le_bytes())?; // format: float
        file.write_all(&channels.to_le_bytes())?;
        file.write_all(&sample_rate.to_le_bytes())?;
        let byte_rate = sample_rate * channels as u32 * 4;
        file.write_all(&byte_rate.to_le_bytes())?;
        file.write_all(&(channels * 4).to_le_bytes())?; // block align
        file.write_all(&32u16.to_le_bytes())?; // bits per sample
        file.write_all(b"data")?;
        file.write_all(&0u32.to_le_bytes())?; // data size (placeholder)
        Ok(WavWriter {
            file_path,
            data_size: 0,
            sample_rate,
            channels,
        })
    }

    fn write_raw(&mut self, data: &[u8]) {
        let mut file = File::create(&self.file_path).ok();
        if let Some(mut file) = file {
            if let Err(e) = file.write_all(data) {
                log::error!("WAV write error: {}", e);
            } else {
                self.data_size += data.len() as u64;
            }
        }
    }

    fn finalize(self) -> Result<(), std::io::Error> {
        let mut file = File::create(&self.file_path)?;
        // Update RIFF chunk size
        file.seek(SeekFrom::Start(4))?;
        let file_size = (self.data_size + 36) as u32;
        file.write_all(&file_size.to_le_bytes())?;
        // Update data chunk size
        file.seek(SeekFrom::Start(40))?;
        file.write_all(&(self.data_size as u32).to_le_bytes())?;
        file.flush()?;
        Ok(())
    }
}
