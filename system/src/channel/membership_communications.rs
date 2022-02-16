use std::net::SocketAddr;
use tokio::sync::mpsc;

pub type MembershipCommunicationsReceiver = mpsc::Receiver<MembershipCommunicationsMessage>;
pub type MembershipCommunicationsSender = mpsc::Sender<MembershipCommunicationsMessage>;

#[derive(Clone, Debug)]
pub enum MembershipCommunicationsMessage {
    Send(Vec<u8>, SocketAddr),
    Shutdown,
}

pub async fn send_message(
    membership_communications: &MembershipCommunicationsSender,
    bytes: Vec<u8>,
    origin: SocketAddr,
) -> Result<(), Box<dyn std::error::Error>> {
    // let (request, response) = oneshot::channel();

    membership_communications
        .send(MembershipCommunicationsMessage::Send(bytes, origin))
        .await?;

    Ok(())
}
