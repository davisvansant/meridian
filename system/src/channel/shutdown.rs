use tokio::sync::broadcast;

pub type ShutdownReceiver = broadcast::Receiver<bool>;
pub type ShutdownSender = broadcast::Sender<bool>;

pub async fn build() -> ShutdownSender {
    let (shutdown_sender, _rx) = broadcast::channel(64);

    shutdown_sender
}

pub async fn shutdown(shutdown: &ShutdownSender) -> Result<(), Box<dyn std::error::Error>> {
    shutdown.send(false)?;

    Ok(())
}
