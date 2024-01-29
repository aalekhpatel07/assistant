#[cfg(test)]
mod tests {

    use assistant::{cache::Cache, listen_on_microphone, tts};
    use std::time::Duration;

    #[tokio::test]
    pub async fn test_client() {
        let cache = Cache::open("redis://localhost:6380").unwrap();
        let client = tts::TextToSpeechClient::new()
            .with_uri("http://localhost:5002")
            .unwrap()
            .with_cache(cache);

        listen_on_microphone().unwrap();
        // let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        // let (stop_speaking_tx, stop_speaking_rx) = tokio::sync::oneshot::channel();

        // let config: cpal::StreamConfig = cpal::StreamConfig {
        //     buffer_size: cpal::BufferSize::Default,
        //     channels: 1,
        //     sample_rate: cpal::SampleRate(22050),
        // };

        let text = "hello how's it going?";

        // tokio::select! {
        //     Ok(data) = client.convert(text) => {
        //         for byte in &data {
        //             _ = tx.send(*byte);
        //         }
        //         drop(tx);
        //     },
        //     _ = tokio::time::sleep(Duration::from_secs(30)) => {
        //         eprintln!("timed out waiting for speech conversion!")
        //     }
        // }

        // tokio::task::spawn(async move {
        //     tokio::time::sleep(Duration::from_secs(45)).await;
        //     stop_speaking_tx.send(()).unwrap();
        // });

        // _ = speak_until_signal(rx, stop_speaking_rx, Some(config)).await;
    }
}


fn main() {
    
}