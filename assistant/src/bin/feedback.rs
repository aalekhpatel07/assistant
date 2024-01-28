use assistant::{listen_until_signal, speak_until_signal};
use std::time::Duration;
use tokio::sync::mpsc::unbounded_channel;
use tokio::sync::oneshot::channel;

#[tokio::main]
pub async fn main() {
    let (tx, rx) = unbounded_channel::<f32>();
    let (stop_listening_tx, stop_listening_rx) = channel();
    let (stop_speaking_tx, stop_speaking_rx) = channel();

    let handle = tokio::task::spawn(async move {
        tokio::time::sleep(Duration::from_secs(10)).await;
        stop_listening_tx.send(()).unwrap();
        stop_speaking_tx.send(()).unwrap();
    });

    let joined = tokio::join!(
        listen_until_signal(tx, stop_listening_rx, None),
        speak_until_signal(rx, stop_speaking_rx, None),
        handle
    );

    joined.0.unwrap();
    joined.1.unwrap();
    joined.2.unwrap();
}
