//! ScreenCaptureKit audio capture for macOS 13.0+
//!
//! Captures system audio output and writes to WAV file.

use std::fs::File;
use std::io::{Seek, SeekFrom, Write};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use screencapturekit::cm::CMSampleBufferDataBufferExt;
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
        log::info!("Audio capture thread exited");
    });
}

/// Stop the current audio capture.
pub fn stop_audio_capture() {
    if CAPTURING.load(Ordering::SeqCst) {
        log::info!("Stop audio capture requested");
        CAPTURING.store(false, Ordering::SeqCst);
    }
}

/// Check if capture is active.
pub fn is_capturing() -> bool {
    CAPTURING.load(Ordering::SeqCst)
}

fn run_capture(file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
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
    let mut wav_writer = WavWriter::new(&wav_path, 48000, 2)?;

    // Use ONE shared flag — the global CAPTURING
    let capturing_ref = Arc::new(&CAPTURING as *const AtomicBool);

    let mut stream = SCStream::new(&filter, &config);

    stream.add_output_handler(
        move |sample: CMSampleBuffer, of_type: SCStreamOutputType| {
            // Check global flag directly
            if !CAPTURING.load(Ordering::SeqCst) {
                return;
            }
            if of_type == SCStreamOutputType::Audio {
                // Try multiple ways to get audio data
                if let Some(buffer_list) = sample.audio_buffer_list() {
                    for i in 0..buffer_list.num_buffers() {
                        if let Some(buffer) = buffer_list.get(i) {
                            wav_writer.write_raw(buffer.data());
                        }
                    }
                } else if let Some(data_buffer) = sample.data_buffer_local() {
                    wav_writer.write_raw(data_buffer.data());
                }
            }
        },
        SCStreamOutputType::Audio,
    );

    stream.start_capture()?;
    log::info!("Audio capture stream started");

    // Wait for stop signal with timeout check
    while CAPTURING.load(Ordering::SeqCst) {
        std::thread::sleep(std::time::Duration::from_millis(50));
    }

    log::info!("Stopping stream...");

    // Give handler time to see the flag change
    std::thread::sleep(std::time::Duration::from_millis(100));

    stream.stop_capture()?;
    log::info!("Audio capture stream stopped");

    // Finalize WAV
    wav_writer.finalize()?;
    log::info!("WAV saved: {}", wav_path.display());

    // Convert to M4A
    let m4a_path = path.with_extension("m4a");
    let status = std::process::Command::new("afconvert")
        .args(&[
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
        std::fs::remove_file(&wav_path)?;
        log::info!("M4A saved: {}", m4a_path.display());
    } else {
        log::warn!("afconvert failed, keeping WAV: {}", wav_path.display());
    }

    Ok(())
}

struct WavWriter {
    file: File,
    data_size: u64,
    sample_rate: u32,
    channels: u16,
}

impl WavWriter {
    fn new(path: &Path, sample_rate: u32, channels: u16) -> Result<Self, std::io::Error> {
        let mut file = File::create(path)?;
        file.write_all(b"RIFF")?;
        file.write_all(&0u32.to_le_bytes())?;
        file.write_all(b"WAVE")?;
        file.write_all(b"fmt ")?;
        file.write_all(&16u32.to_le_bytes())?;
        file.write_all(&3u16.to_le_bytes())?; // float
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
            sample_rate,
            channels,
        })
    }

    fn write_raw(&mut self, data: &[u8]) {
        if let Err(e) = self.file.write_all(data) {
            log::error!("WAV write error: {}", e);
        } else {
            self.data_size += data.len() as u64;
        }
    }

    fn finalize(mut self) -> Result<(), std::io::Error> {
        self.file.seek(SeekFrom::Start(4))?;
        let file_size = (self.data_size + 36) as u32;
        self.file.write_all(&file_size.to_le_bytes())?;
        self.file.seek(SeekFrom::Start(40))?;
        self.file
            .write_all(&(self.data_size as u32).to_le_bytes())?;
        self.file.flush()?;
        Ok(())
    }
}
