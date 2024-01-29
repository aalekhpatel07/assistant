#[cfg(test)]
mod tests {

    use assistant::{cache::Cache, listen_on_microphone, play_on_speaker, tts};
    use std::time::Duration;

    #[tokio::test]
    pub async fn test_client() {
        let cache = Cache::open("redis://localhost:6380").unwrap();
        let client = tts::TextToSpeechClient::new()
            .with_uri("http://localhost:5002")
            .unwrap()
            .with_cache(cache);

        // let data = listen_on_microphone().unwrap();
        let text = "very nice. i like";
        let (config, data) = client.convert(text).await.unwrap();
        // let (config, data) = listen_on_microphone(Default::default()).unwrap();
        play_on_speaker(&config, data);

    }
}


fn main() {
    
}