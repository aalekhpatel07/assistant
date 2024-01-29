use std::io::Cursor;

use crate::stream::record_audio;

pub(crate) fn parse_wav_from_response(contents: bytes::Bytes) -> Vec<i16> {
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

    samples.into_iter().collect::<Result<Vec<_>, _>>().unwrap()
}

#[derive(Debug, Clone, Copy)]
pub struct StreamSettings {
    /// value used for pause detection, a pause is detected when the amplitude is less than this
    pub silence_level: i32,
    /// show the amplitude values on stdout (helps you to find your silence level)
    pub show_amplitudes: bool,
    /// seconds of silence indicating end of speech
    pub pause_length_millis: u32,
}

impl StreamSettings {
    /// Create a new configuration for a mic stream
    pub fn new(silence_level: i32, show_amplitudes: bool, pause_length_millis: u32) -> Self {
        Self {
            silence_level,
            show_amplitudes,
            pause_length_millis,
        }
    }
}

impl Default for StreamSettings {
    fn default() -> Self {
        Self {
            silence_level: 200,
            show_amplitudes: true,
            pause_length_millis: 1000,
        }
    }
}


pub fn transcribe(config: StreamSettings) -> anyhow::Result<Vec<i16>> {
    let pause_length_ms = (config.pause_length_millis / 1000) as f32;
    record_audio(
        config.silence_level,
        config.show_amplitudes,
        pause_length_ms
    )
}

pub fn listen_on_microphone() -> anyhow::Result<Vec<i16>> {
    transcribe(Default::default())
}

// pub(crate) async fn parse_audio() {
//     let mut microphone = Microphone::default();
//     let mut stream: MicrophoneStream<'_, fon::mono::Mono32> = microphone.record().await;
//     let foo = stream.collect::<Vec<_>>();
    
//     // stream.;
// }