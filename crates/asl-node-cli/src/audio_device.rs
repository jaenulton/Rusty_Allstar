#![allow(clippy::needless_range_loop)]
//! Host sound card audio I/O using CPAL for `asl-node-cli`.
//!
//! Captures microphone audio, downsamples/converts to 8 kHz 16-bit mono PCM,
//! and plays 8 kHz μ-law decoded PCM to the default output device.

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Stream, StreamConfig};
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};
use tracing::{error, info, warn};

pub struct AudioSubsystem {
    _input_stream: Option<Stream>,
    _output_stream: Option<Stream>,
    _running: Arc<AtomicBool>,
}

impl AudioSubsystem {
    /// Initialize desktop audio with channels to send microphone PCM frames
    /// to the IAX2 streamer and receive IAX2 speaker PCM frames to play.
    pub fn start(
        mic_tx: tokio::sync::mpsc::Sender<[i16; 160]>,
        mut speaker_rx: tokio::sync::mpsc::Receiver<[i16; 160]>,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let host = cpal::default_host();
        let running = Arc::new(AtomicBool::new(true));

        // 1. Setup Output Device (Speaker / Headphones)
        let output_stream = if let Some(device) = host.default_output_device() {
            let config: StreamConfig = device.default_output_config()?.into();
            let sample_rate = config.sample_rate.0;
            let channels = config.channels as usize;

            info!(
                "Configured audio output: '{}' @ {} Hz ({} channels)",
                device.name().unwrap_or_default(),
                sample_rate,
                channels
            );

            let audio_buffer =
                Arc::new(Mutex::new(Vec::<f32>::with_capacity(sample_rate as usize)));
            let buf_clone = audio_buffer.clone();

            // Background task feeding speaker frames to ring buffer
            tokio::spawn(async move {
                while let Some(frame) = speaker_rx.recv().await {
                    let ratio = (sample_rate as f32 / 8000.0).round().max(1.0) as usize;
                    let mut lock = buf_clone.lock().unwrap();
                    // Keep buffer under 0.5 sec to prevent latency buildup
                    if lock.len() > sample_rate as usize / 2 {
                        lock.clear();
                    }
                    for i in 0..160 {
                        let sample_f32 = frame[i] as f32 / 32768.0;
                        for _ in 0..ratio {
                            lock.push(sample_f32);
                        }
                    }
                }
            });

            let play_buf = audio_buffer.clone();
            let stream = device.build_output_stream(
                &config,
                move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                    let mut lock = play_buf.lock().unwrap();
                    for frame in data.chunks_mut(channels) {
                        let s = if !lock.is_empty() {
                            lock.remove(0)
                        } else {
                            0.0
                        };
                        for sample in frame.iter_mut() {
                            *sample = s;
                        }
                    }
                },
                |err| error!("Audio output error: {:?}", err),
                None,
            )?;
            stream.play()?;
            Some(stream)
        } else {
            warn!("No default audio output device found.");
            None
        };

        // 2. Setup Input Device (Microphone)
        let input_stream = if let Some(device) = host.default_input_device() {
            let config: StreamConfig = device.default_input_config()?.into();
            let in_sample_rate = config.sample_rate.0;
            let in_channels = config.channels as usize;

            info!(
                "Configured audio input: '{}' @ {} Hz ({} channels)",
                device.name().unwrap_or_default(),
                in_sample_rate,
                in_channels
            );

            let mic_buffer = Arc::new(Mutex::new(Vec::<f32>::with_capacity(
                in_sample_rate as usize,
            )));
            let mic_buf_clone = mic_buffer.clone();

            tokio::spawn(async move {
                let downsample_ratio = (in_sample_rate as f32 / 8000.0).round().max(1.0) as usize;
                let mut pcm_buf = [0i16; 160];
                let mut count = 0;
                let mut sample_accum = 0.0f32;
                let mut accum_count = 0;

                loop {
                    let next_sample = {
                        let mut lock = mic_buf_clone.lock().unwrap();
                        if !lock.is_empty() {
                            Some(lock.remove(0))
                        } else {
                            None
                        }
                    };

                    if let Some(s) = next_sample {
                        sample_accum += s;
                        accum_count += 1;
                        if accum_count >= downsample_ratio {
                            let avg = sample_accum / (accum_count as f32);
                            pcm_buf[count] = (avg * 32767.0).clamp(-32768.0, 32767.0) as i16;
                            count += 1;
                            sample_accum = 0.0;
                            accum_count = 0;

                            if count >= 160 {
                                if mic_tx.send(pcm_buf).await.is_err() {
                                    break;
                                }
                                count = 0;
                            }
                        }
                    } else {
                        tokio::time::sleep(tokio::time::Duration::from_millis(2)).await;
                    }
                }
            });

            let stream_buf = mic_buffer.clone();
            let stream = device.build_input_stream(
                &config,
                move |data: &[f32], _: &cpal::InputCallbackInfo| {
                    let mut lock = stream_buf.lock().unwrap();
                    for frame in data.chunks(in_channels) {
                        if lock.len() < in_sample_rate as usize {
                            lock.push(frame[0]);
                        }
                    }
                },
                |err| error!("Audio input error: {:?}", err),
                None,
            )?;
            stream.play()?;
            Some(stream)
        } else {
            warn!("No default audio input device found.");
            None
        };

        Ok(Self {
            _input_stream: input_stream,
            _output_stream: output_stream,
            _running: running,
        })
    }
}
