use tonic::{Code, Request, Response, Status};

pub use crate::meridian_membership_v010::communications_server::{
    Communications, CommunicationsServer,
};

use crate::{JoinClusterRequest, JoinClusterResponse};

use crate::channels::ChannelMembershipReceiveAction;
use crate::channels::ChannelMembershipSendGrpcAction;

use crate::channels::MembershipReceiveAction;
use crate::channels::MembershipSendGrpcAction;

pub struct ExternalMembershipGrpcServer {
    receive_membership_action: ChannelMembershipSendGrpcAction,
    send_membership_action: ChannelMembershipReceiveAction,
}

impl ExternalMembershipGrpcServer {
    pub async fn init(
        receive_membership_action: ChannelMembershipSendGrpcAction,
        send_membership_action: ChannelMembershipReceiveAction,
    ) -> Result<ExternalMembershipGrpcServer, Box<dyn std::error::Error>> {
        Ok(ExternalMembershipGrpcServer {
            receive_membership_action,
            send_membership_action,
        })
    }

    async fn check_incoming_request(request: &JoinClusterRequest) -> bool {
        request.id.is_empty()
            | request.address.is_empty()
            | request.client_port.is_empty()
            | request.cluster_port.is_empty()
            | request.membership_port.is_empty()
    }
}

#[tonic::async_trait]
impl Communications for ExternalMembershipGrpcServer {
    async fn join_cluster(
        &self,
        request: Request<JoinClusterRequest>,
    ) -> Result<Response<JoinClusterResponse>, Status> {
        println!("incoming join request ... {:?}", &request);
        if Self::check_incoming_request(request.get_ref()).await {
            let message = String::from("Received empty request!");
            let status = Status::new(Code::FailedPrecondition, message);
            Err(status)
        } else {
            if let Err(error) =
                self.send_membership_action
                    .send(MembershipReceiveAction::JoinClusterRequest(
                        request.into_inner(),
                    ))
            {
                println!("{:?}", error);
            };

            let mut receiver = self.receive_membership_action.subscribe();

            match receiver.recv().await {
                Ok(MembershipSendGrpcAction::JoinClusterResponse(response)) => {
                    Ok(Response::new(response))
                }
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
