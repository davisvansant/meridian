use std::net::SocketAddr;
// use tokio::sync::mpsc;
use tokio::sync::broadcast;

// pub type MembershipCommunicationsReceiver = mpsc::Receiver<MembershipCommunicationsMessage>;
// pub type MembershipCommunicationsSender = mpsc::Sender<MembershipCommunicationsMessage>;
pub type MembershipCommunicationsReceiver = broadcast::Receiver<MembershipCommunicationsMessage>;
pub type MembershipCommunicationsSender = broadcast::Sender<MembershipCommunicationsMessage>;

#[derive(Clone, Debug)]
pub enum MembershipCommunicationsMessage {
    Send(Vec<u8>, SocketAddr),
    // Shutdown,
}

pub async fn send_message(
    membership_communications: &MembershipCommunicationsSender,
    bytes: Vec<u8>,
    origin: SocketAddr,
) -> Result<(), Box<dyn std::error::Error>> {
    // let (request, response) = oneshot::channel();

    // membership_communications
    //     .send(MembershipCommunicationsMessage::Send(bytes, origin))
    //     .await?;
    membership_communications.send(MembershipCommunicationsMessage::Send(bytes, origin))?;

    Ok(())
}
