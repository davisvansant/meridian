use std::net::SocketAddr;
use tokio::sync::{mpsc, oneshot};

pub type MembershipMaintenanceShutdown = mpsc::Receiver<bool>;
pub type MembershipMaintenanceReceiver = mpsc::Receiver<(
    MembershipMaintenanceRequest,
    oneshot::Sender<MembershipMaintenanceResponse>,
)>;
pub type MembershipMaintenanceSender = mpsc::Sender<(
    MembershipMaintenanceRequest,
    oneshot::Sender<MembershipMaintenanceResponse>,
)>;

#[derive(Clone, Debug)]
pub enum MembershipMaintenanceRequest {
    Message((Vec<u8>, SocketAddr)),
    Shutdown,
}

#[derive(Clone, Debug)]
pub enum MembershipMaintenanceResponse {}

pub async fn shutdown_membership_maintenance(
    membership_maintenance: &MembershipMaintenanceSender,
) -> Result<(), Box<dyn std::error::Error>> {
    let (request, _response) = oneshot::channel();

    membership_maintenance
        .send((MembershipMaintenanceRequest::Shutdown, request))
        .await?;

    Ok(())
}
