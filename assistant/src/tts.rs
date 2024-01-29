use crate::cache::Cache;
use crate::parse_wav_from_response;
use anyhow::bail;
use bytes::Bytes;
use reqwest::Url;
use std::env::var;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct TextToSpeechClient {
    client: reqwest::Client,
    base_uri: Option<Url>,
    cache: Option<Cache>,
}

impl Default for TextToSpeechClient {
    fn default() -> Self {
        Self::new()
    }
}

impl TextToSpeechClient {
    pub fn new() -> Self {
        Self {
            client: reqwest::ClientBuilder::default()
                .timeout(Duration::from_secs(60))
                .build()
                .expect("failed to build reqwest Client"),
            base_uri: var("TTS_URL")
                .map(|v| v.parse::<Url>().ok())
                .unwrap_or(None),
            cache: None,
        }
    }

    pub fn with_uri(self, uri: &str) -> anyhow::Result<Self> {
        let base_uri: Url = uri.parse()?;
        Ok(Self {
            base_uri: Some(base_uri),
            ..self
        })
    }

    pub fn with_cache(self, cache: Cache) -> Self {
        Self {
            cache: Some(cache),
            ..self
        }
    }

    /// Call the Text-to-Speech inference server and get the audio data.
    async fn get_wav(&self, text: &str) -> anyhow::Result<Bytes> {
        let Some(ref base_uri) = self.base_uri else {
            bail!("No base_uri provided");
        };

        let api_endpoint = format!("{}/api/tts", base_uri);

        let request = self
            .client
            .get(api_endpoint)
            .query(&[("text", text)])
            .build()?;

        let client = self.client.clone();

        let response = client.execute(request).await?;
        let contents = response.bytes().await?;

        Ok(contents)
    }

    /// Attempt to convert the given text into its audio version.
    pub async fn convert(&self, text: &str) -> anyhow::Result<Vec<i16>> {
        if let Some(cache) = &self.cache {
            if let Ok(Some(contents)) = cache.load_wav_file(text).await {
                return Ok(contents);
            }
        }

        let contents = self.get_wav(text).await?;
        let data = parse_wav_from_response(contents);

        if let Some(cache) = &self.cache {
            if let Err(err) = cache.store_wav_file(text, &data).await {
                eprintln!("couldn't cache the transcription this time, whoops: {err}");
            }
        }

        Ok(data)
    }
}
