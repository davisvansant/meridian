use crate::grpc::{Code, Request, Response, Status};
pub use crate::meridian_membership_v010::communications_server::{
    Communications, CommunicationsServer,
};
use crate::runtime::sync::membership_receive_task::ChannelMembershipReceiveTask;
use crate::runtime::sync::membership_receive_task::MembershipReceiveTask;
use crate::runtime::sync::membership_send_grpc_task::ChannelMembershipSendGrpcTask;
use crate::runtime::sync::membership_send_grpc_task::MembershipSendGrpcTask;
use crate::{JoinClusterRequest, JoinClusterResponse};

pub struct ExternalMembershipGrpcServer {
    receive_membership_task: ChannelMembershipSendGrpcTask,
    send_membership_task: ChannelMembershipReceiveTask,
}

impl ExternalMembershipGrpcServer {
    pub async fn init(
        receive_membership_task: ChannelMembershipSendGrpcTask,
        send_membership_task: ChannelMembershipReceiveTask,
    ) -> Result<ExternalMembershipGrpcServer, Box<dyn std::error::Error>> {
        Ok(ExternalMembershipGrpcServer {
            receive_membership_task,
            send_membership_task,
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
                self.send_membership_task
                    .send(MembershipReceiveTask::JoinClusterRequest(
                        request.into_inner(),
                    ))
            {
                println!("{:?}", error);
            };

            let mut receiver = self.receive_membership_task.subscribe();

            match receiver.recv().await {
                Ok(MembershipSendGrpcTask::JoinClusterResponse(response)) => {
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
