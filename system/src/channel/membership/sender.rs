use std::net::SocketAddr;
use tokio::sync::broadcast;

// pub type DisseminationReceiver = broadcast::Receiver<Dissemination>;
pub type DisseminationSender = broadcast::Sender<Dissemination>;

#[derive(Clone, Debug)]
pub enum Dissemination {
    Message(Vec<u8>, SocketAddr),
}

impl Dissemination {
    pub async fn build() -> DisseminationSender {
        let (dissemination_sender, _) = broadcast::channel::<Dissemination>(64);

        dissemination_sender
    }
}
