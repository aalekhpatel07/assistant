//! Does the actual recording and white noise cleaning
//! Pretty much copypasta of [`ds-transcriber's stream.rs`]
//! [`ds-transcriber's stream.rs`](https://github.com/rtkay123/ds-transcriber/blob/master/src/stream.rs)

use anyhow::{bail, Result};
#[cfg(feature = "denoise")]
use nnnoiseless::DenoiseState;
use std::{
    ops::Neg,
    sync::mpsc::{channel, Receiver},
    time::Instant,
};
// use tracing::{error, warn, info, trace};

use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    Device, SampleRate, SupportedStreamConfig,
};

const SAMPLE_RATE: u32 = 16000;

///
/// # Input device configuration
/// Gets data ready to begin recording
/// 
pub struct StreamConfig {
    device: Device,
    config: SupportedStreamConfig,
    silence_level: i32,
}

impl StreamConfig {
    pub fn new(silence_level: i32) -> anyhow::Result<Self> {
        let host = cpal::default_host();
        let device = host.default_input_device().expect("No input device found.");
        match device.default_input_config() {
            Ok(config) => Ok(StreamConfig {
                config,
                device,
                silence_level,
            }),
            Err(e) => {
                bail!("{e}")
            }
        }
    }

    pub fn device(&self) -> &Device {
        &self.device
    }

    pub fn supported_config(&self) -> &SupportedStreamConfig {
        &self.config
    }

    pub fn silence_level(&self) -> i32 {
        self.silence_level
    }
}

///
/// # Recording Audio
/// This function handles the audio recording process, parameters are obtained from `ds_transcriber::transcriber::StreamSettings`
///
pub fn record_audio(
    silence_level: i32,
    show_amplitude: bool,
    pause_length: f32,
) -> Result<(cpal::StreamConfig, Vec<i16>)> {
    let config = StreamConfig::new(silence_level)?;
    let (sound_sender, sound_receiver) = channel();
    let device = config.device();
    let stream_config = config.supported_config().config();
    println!("record_audio stream_config: {:?}", stream_config);
    let stream = device.build_input_stream(
        &stream_config,
        move |data: &[f32], _: &_| {
            if let Err(_e) = sound_sender.send(data.to_owned()) {
                // error!("{}", e);
            }
        },
        move |_err| {},
        None
    )?;

    stream.play()?;

    let denoised_stream = start(
        &sound_receiver,
        config.silence_level(),
        show_amplitude,
        pause_length,
    )?;

    let audio_buf = denoised_stream
        .into_iter()
        .map(|a| (a * i16::MAX as f32) as i16)
        .collect::<Vec<_>>();

    Ok((stream_config, audio_buf))
}


fn start(
    sound_receiver: &Receiver<Vec<f32>>,
    silence_level: i32,
    show_amplitude: bool,
    pause_length: f32,
) -> Result<Vec<f32>> {
    let mut silence_start = None;
    let mut sound_from_start_till_pause = vec![];

    loop {
        let small_sound_chunk = sound_receiver.recv()?;
        sound_from_start_till_pause.extend(&small_sound_chunk);
        let sound_as_ints = small_sound_chunk.iter().map(|f| (*f * 1000.0) as i32);
        let max_amplitude = sound_as_ints.clone().max().unwrap_or(0);
        let min_amplitude = sound_as_ints.clone().min().unwrap_or(0);
        if show_amplitude {
            eprintln!("Min is {}, Max is {}", min_amplitude, max_amplitude);
        }
        let silence_detected = max_amplitude < silence_level && min_amplitude > silence_level.neg();
        if silence_detected {
            match silence_start {
                None => silence_start = Some(Instant::now()),
                Some(s) => {
                    if s.elapsed().as_secs_f32() > pause_length {
                        #[cfg(feature = "denoise")]
                        {
                            // trace!("denoising stream");
                            return Ok(denoise(sound_from_start_till_pause));
                        }
                        #[cfg(not(feature = "denoise"))]
                        {
                            return Ok(sound_from_start_till_pause);
                        }
                    }
                }
            }
        } else {
            silence_start = None;
        }
    }
}

#[cfg(feature = "denoise")]
fn denoise(sound_from_start_till_pause: Vec<f32>) -> Vec<f32> {
    let mut output = Vec::new();
    let mut out_buf = [0.0; DenoiseState::FRAME_SIZE];
    let mut denoise = DenoiseState::new();
    let mut first = true;
    for chunk in sound_from_start_till_pause.chunks_exact(DenoiseState::FRAME_SIZE) {
        denoise.process_frame(&mut out_buf[..], chunk);
        if !first {
            output.extend_from_slice(&out_buf[..]);
        }
        first = false;
    }
    output
}