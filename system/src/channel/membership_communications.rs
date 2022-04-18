use std::net::SocketAddr;
use tokio::sync::broadcast;

pub type MembershipCommunicationsSender = broadcast::Sender<MembershipCommunicationsMessage>;

#[derive(Clone, Debug)]
pub enum MembershipCommunicationsMessage {
    Send(Vec<u8>, SocketAddr),
}

pub async fn send_message(
    membership_communications: &MembershipCommunicationsSender,
    bytes: &[u8],
    origin: SocketAddr,
) -> Result<(), Box<dyn std::error::Error>> {
    membership_communications.send(MembershipCommunicationsMessage::Send(
        bytes.to_vec(),
        origin,
    ))?;

    Ok(())
}
