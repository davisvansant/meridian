use std::net::SocketAddr;
use tokio::sync::mpsc;

pub type MembershipCommunicationsReceiver = mpsc::Receiver<MembershipCommunicationsMessage>;
pub type MembershipCommunicationsSender = mpsc::Sender<MembershipCommunicationsMessage>;

#[derive(Clone, Debug)]
pub enum MembershipCommunicationsMessage {
    Send(Vec<u8>, SocketAddr),
    Shutdown,
}
