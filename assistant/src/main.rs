use std::time::Duration;

use assistant::{
    cache::Cache, 
    listen_on_microphone, 
    play_on_speaker, 
    stt, 
    tts, 
    build_wav_from_audio, 
    bot::OpenAIChatCompletion
};

#[tokio::main]
pub async fn main() {
    let cache = Cache::open("redis://localhost:6380").unwrap();
    let tts_client = tts::TextToSpeechClient::new()
        .with_uri("http://localhost:5002")
        .unwrap()
        .with_cache(cache);

    let stt_client = stt::SpeechToTextClient::new()
        .with_uri("http://localhost:9003")
        .unwrap();

    let mut chat_completion = OpenAIChatCompletion::new(None).unwrap();

    loop {
        println!("Listening for input...");
        let (config, captured_data) = listen_on_microphone(Default::default()).unwrap();
        let wav_data = build_wav_from_audio(config.clone(), &captured_data).unwrap();
        println!("Recorded wav file has {} bytes", wav_data.len());
        if wav_data.len() <= 500_000 {
            println!("Recorded wav file too small. Probably a lotta background noise.");
            continue;
        }
        let text = stt_client.transcribe(wav_data, Some("en")).await.unwrap();
        println!("Listened to text: {text}");
        if text.trim().is_empty() {
            continue;
        }

        let Ok(chat_response) = chat_completion.speak(&text) else {
            continue;
        };
        println!("Chat response: {chat_response}");
        let (config, data) = tts_client.convert(&chat_response).await.unwrap();
        play_on_speaker(&config, data);
        println!("Finished playing chatgpt output to speaker.");
    }

}