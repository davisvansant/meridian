use tokio::sync::broadcast::Sender;
use tonic::{Code, Request, Response, Status};

pub use crate::meridian_membership_v010::communications_server::{
    Communications, CommunicationsServer,
};

use crate::{JoinClusterRequest, JoinClusterResponse};

pub struct ExternalMembershipGrpcServer {
    membership_receive_actions: Sender<JoinClusterResponse>,
    membership_send_actions: Sender<JoinClusterRequest>,
}

impl ExternalMembershipGrpcServer {
    pub async fn init(
        membership_receive_actions: Sender<JoinClusterResponse>,
        membership_send_actions: Sender<JoinClusterRequest>,
    ) -> Result<ExternalMembershipGrpcServer, Box<dyn std::error::Error>> {
        Ok(ExternalMembershipGrpcServer {
            membership_receive_actions,
            membership_send_actions,
        })
    }
}

#[tonic::async_trait]
impl Communications for ExternalMembershipGrpcServer {
    async fn join_cluster(
        &self,
        request: Request<JoinClusterRequest>,
    ) -> Result<Response<JoinClusterResponse>, Status> {
        println!("{:?}", &request);
        if request.get_ref().node_id.is_empty()
            | request.get_ref().address.is_empty()
            | request.get_ref().port.is_empty()
        {
            let message = String::from("Received empty request!");
            let status = Status::new(Code::FailedPrecondition, message);
            Err(status)
        } else {
            if let Err(error) = self.membership_send_actions.send(request.into_inner()) {
                println!("{:?}", error);
            };

            let mut receiver = self.membership_receive_actions.subscribe();

            match receiver.recv().await {
                Ok(response) => Ok(Response::new(response)),
                Err(error) => {
                    let message = error.to_string();
                    let status = Status::new(Code::NotFound, message);
                    Err(status)
                }
            }
        }
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     #[tokio::test(flavor = "multi_thread")]
//     async fn init() -> Result<(), Box<dyn std::error::Error>> {
//         let test_external_membership_grpc_server = ExternalMembershipGrpcServer::init().await;
//         assert!(test_external_membership_grpc_server.is_ok());
//         Ok(())
//     }
// }
