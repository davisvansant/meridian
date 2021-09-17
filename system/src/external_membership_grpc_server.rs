// use tokio::sync::broadcast::Sender;
use tonic::{Code, Request, Response, Status};

pub use crate::meridian_membership_v010::communications_server::{
    Communications, CommunicationsServer,
};

use crate::{JoinClusterRequest, JoinClusterResponse};

pub struct ExternalMembershipGrpcServer {
    // receive_actions: Sender<Actions>,
// send_actions: Sender<Actions>,
}

impl ExternalMembershipGrpcServer {
    pub async fn init(// receive_actions: Sender<Actions>,
    // send_actions: Sender<Actions>,
    ) -> Result<ExternalMembershipGrpcServer, Box<dyn std::error::Error>> {
        Ok(ExternalMembershipGrpcServer {
            // receive_actions,
            // send_actions,
        })
    }
}

#[tonic::async_trait]
impl Communications for ExternalMembershipGrpcServer {
    async fn join_cluster(
        &self,
        request: Request<JoinClusterRequest>,
    ) -> Result<Response<JoinClusterResponse>, Status> {
        let response = JoinClusterResponse {
            success: String::from("true"),
            details: String::from("node successfully joined cluster!"),
            members: Vec::with_capacity(0),
        };

        Ok(Response::new(response))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test(flavor = "multi_thread")]
    async fn init() -> Result<(), Box<dyn std::error::Error>> {
        let test_external_membership_grpc_server = ExternalMembershipGrpcServer::init().await;
        assert!(test_external_membership_grpc_server.is_ok());
        Ok(())
    }
}
