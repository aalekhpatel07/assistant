use std::io::{BufReader, BufWriter, Cursor};

use bytes::{buf, Buf};
use cpal::{traits::{HostTrait, DeviceTrait, StreamTrait}, SampleRate, SizedSample};
use hound;
use tokio::sync::oneshot::error::TryRecvError;
use crate::errors::AssistantError;


pub(crate) fn convert_wav(contents: bytes::Bytes) -> Vec<i16>
{
    let reader = Cursor::new(contents);
    let buf_reader = hound::WavReader::new(reader).unwrap();

    let spec = buf_reader.spec();
    
    eprintln!("spec: {:#?}", spec);
    eprintln!("duration: {:.3}s", buf_reader.duration() as f32 / buf_reader.spec().sample_rate as f32);
    eprintln!("len: {:}", buf_reader.len());

    let samples = buf_reader.into_samples::<i16>();

    samples.into_iter().collect::<Result<Vec<_>, _>>().unwrap()
}


pub async fn listen_until_signal<T>(
    data_tx: tokio::sync::mpsc::UnboundedSender<T>,
    stop_listening_rx: tokio::sync::oneshot::Receiver<()>,
    config: Option<cpal::StreamConfig>
) -> Result<(), AssistantError> 
where
    T: SizedSample + Sync + Send + 'static
{

    let host = cpal::default_host();
    let device = host.default_input_device().ok_or_else(|| cpal::StreamError::DeviceNotAvailable)?;
    let config = config.unwrap_or_else(|| device.default_input_config().expect("failed to get default input config").into());

    let on_input_data = move |data: &[T], _info: &cpal::InputCallbackInfo| {
        for &sample in data {
            if let Err(err) = data_tx.send(sample) {
                panic!("dropped receiver somewhere?: {err:?}")
            }
        }
    };

    let err_fn = move |err| {
        eprintln!("failed to read from input: {err:?}");
    };

    let stream = device.build_input_stream(
        &config, 
        on_input_data,
        err_fn,
        None
    )?;
    stream.play()?;
    
    if let Err(err) = stop_listening_rx.await {
        eprintln!("Failed to recv stop signal: {}", err);
    }
    Ok(())
}


pub async fn speak_until_signal<T>(
    mut data_rx: tokio::sync::mpsc::UnboundedReceiver<T>,
    stop_speaking_rx: tokio::sync::oneshot::Receiver<()>,
    config: Option<cpal::StreamConfig>
) -> Result<(), AssistantError> 
where
    T: SizedSample + Sync + Send + 'static + Default
{

    let host = cpal::default_host();
    let device = host.default_output_device().ok_or_else(|| cpal::StreamError::DeviceNotAvailable)?;

    let config = config.unwrap_or_else(|| device.default_output_config().expect("failed to get default output config").into());
    println!("speaker config: {:#?}", config);

    let (no_input_tx, mut no_input_rx) = tokio::sync::mpsc::unbounded_channel();

    let on_output_data = move |data: &mut [T], _info: &cpal::OutputCallbackInfo| {
        for sample in data.iter_mut() {
            match data_rx.try_recv() {
                Ok(val) => {
                    *sample = val;
                },
                Err(err) => {
                    match err {
                        tokio::sync::mpsc::error::TryRecvError::Disconnected => {
                            _ = no_input_tx.send(());
                            return;
                        },
                        tokio::sync::mpsc::error::TryRecvError::Empty => {
                            *sample = T::default();
                        }
                    }
                }
            }
        }
    };

    let err_fn = move |err| {
        eprintln!("failed to write to output: {err:?}");
    };

    let stream = device.build_output_stream(
        &config, 
        on_output_data,
        err_fn,
        None
    )?;
    stream.play()?;

    tokio::select! {
        _ = stop_speaking_rx => {

        },
        _ = no_input_rx.recv() => {

        }
    }

    Ok(())
}