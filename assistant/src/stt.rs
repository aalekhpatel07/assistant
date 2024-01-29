use reqwest;
use reqwest::Url;
use std::time::Duration;
use std::env::var;


mod listener {
    use std::borrow::Borrow;
    use std::future::{poll_fn, Pending, Ready};
    use std::task::Poll;

}



#[derive(Debug)]
pub struct SpeechToTextClient {
    client: reqwest::Client,
    base_uri: Option<Url>,
}

impl SpeechToTextClient {
    pub async fn new() -> Self {

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
}