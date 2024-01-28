use thiserror::Error;

#[derive(Debug, Error)]
pub enum AssistantError {
    NoInputDeviceAvailable(#[from] cpal::StreamError),
    AudioInputStreamSetupFailed(#[from] cpal::BuildStreamError),
    UnsupportedStream(#[from] cpal::DefaultStreamConfigError),
    FailedToPlayStream(#[from] cpal::PlayStreamError),
}

impl std::fmt::Display for AssistantError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
