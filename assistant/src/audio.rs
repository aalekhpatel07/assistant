use std::{io::Cursor, time::Duration};
use cpal::{traits::{DeviceTrait, HostTrait, StreamTrait}, SampleRate, StreamConfig};

use crate::stream::record_audio;

pub(crate) fn parse_wav_from_response(contents: &[u8]) -> (StreamConfig, Vec<i16>) {
    let reader = Cursor::new(contents);
    let buf_reader = hound::WavReader::new(reader).unwrap();

    let spec = buf_reader.spec();

    eprintln!("spec: {:#?}", spec);
    eprintln!(
        "duration: {:.3}s",
        buf_reader.duration() as f32 / buf_reader.spec().sample_rate as f32
    );
    eprintln!("len: {:}", buf_reader.len());

    let samples = buf_reader.into_samples::<i16>();

    (
        StreamConfig {
            channels: spec.channels,
            sample_rate: SampleRate(spec.sample_rate),
            buffer_size: cpal::BufferSize::Default
        }, 
        samples.into_iter().collect::<Result<Vec<_>, _>>().unwrap()
    )
}

#[derive(Debug, Clone, Copy)]
pub struct StreamSettings {
    /// value used for pause detection, a pause is detected when the amplitude is less than this
    pub silence_level: i32,
    /// show the amplitude values on stdout (helps you to find your silence level)
    pub show_amplitudes: bool,
    /// seconds of silence indicating end of speech
    pub pause_length: Duration,
}

impl StreamSettings {
    /// Create a new configuration for a mic stream
    pub fn new(silence_level: i32, show_amplitudes: bool, pause_length_millis: u32) -> Self {
        Self {
            silence_level,
            show_amplitudes,
            pause_length: Duration::from_millis(pause_length_millis as u64),
        }
    }
}

impl Default for StreamSettings {
    fn default() -> Self {
        Self {
            silence_level: 200,
            show_amplitudes: true,
            pause_length: Duration::from_secs(1),
        }
    }
}


pub fn listen_on_microphone(config: StreamSettings) -> anyhow::Result<(cpal::StreamConfig, Vec<i16>)> {
    let pause_length_ms = (config.pause_length.as_millis() / 1000) as f32;
    record_audio(
        config.silence_level,
        config.show_amplitudes,
        pause_length_ms
    )
}

pub fn play_on_speaker(config: &StreamConfig, data: Vec<i16>) {
    let (complete_tx, complete_rx) = std::sync::mpsc::sync_channel(1);
    println!("playing on speaker: len: {:?}", data.len());
    let mut data_iter = data.into_iter();
    let host = cpal::default_host();
    let device = host.default_output_device().expect("default output device not found");

    let stream = device.build_output_stream(
        &config,
        move |data: &mut [i16], _| {
            for sample in data {
                if let Some(val) = data_iter.next() {
                    *sample = val as i16;
                } else {
                    _ = complete_tx.send(());
                    return;
                }
            }
        },
        |err| {
            println!("failed to write to stream: {err}")
        }, 
        None
    ).unwrap();

    stream.play().unwrap();
    let _ = complete_rx.recv();

}
