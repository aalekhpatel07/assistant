use anyhow::bail;
use reqwest;
use reqwest::Url;
use std::time::Duration;
use std::env::var;


#[derive(Debug)]
pub struct SpeechToTextClient {
    client: reqwest::Client,
    base_uri: Option<Url>,
}

impl SpeechToTextClient {
    pub fn new() -> Self {

        Self {
            client: reqwest::ClientBuilder::default()
                .timeout(Duration::from_secs(60))
                .build()
                .expect("failed to build reqwest Client"),
            base_uri: var("STT_URL")
                .map(|v| v.parse::<Url>().ok())
                .unwrap_or(None),
        }
    }

    pub fn with_uri(self, uri: &str) -> anyhow::Result<Self> {
        let base_uri: reqwest::Url = uri.parse()?;
        Ok(Self {
            base_uri: Some(base_uri),
            ..self
        })
    }

    pub async fn transcribe(
        &self, 
        audio_data: bytes::Bytes,
        language: Option<&str>,
    ) -> anyhow::Result<String> {
        let Some(ref base_uri) = self.base_uri else {
            bail!("No base_uri provided");
        };
        // eprintln!("base_uri: {}", base_uri);
        let api_endpoint = format!("{}asr", base_uri);

        let part = reqwest::multipart::Part::stream(audio_data).file_name("dummy.wav").mime_str("audio/wav")?;
        let form = reqwest::multipart::Form::new().part("audio_file", part);

        let mut query = vec![
            ("encode", "true"), 
            ("task", "transcribe"),
            ("word_timestamps", "false"),
            ("output", "txt")
        ];

        if let Some(lang) = language {
            query.push(("language", lang));
        }

        let request = self
        .client
        .post(api_endpoint)
        .query(&query)
        .multipart(form)
        .build()?;

        // eprintln!("stt request: {:#?}", request);
        match self.client.execute(request)
        .await {
            Ok(resp) => {
                let contents = resp.bytes().await?;
                Ok(String::from_utf8_lossy(&contents).trim().to_string())
            },
            Err(err) => {
                eprintln!("Failed to get response: {err}");
                bail!("{err}")
            }
        }
    }
}