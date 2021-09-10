use tokio::sync::broadcast::Sender;
use tonic::{Code, Request, Response, Status};

use crate::meridian_cluster_v010::{
    AppendEntriesRequest, AppendEntriesResponse, InstallSnapshotRequest, InstallSnapshotResponse,
    RequestVoteRequest, RequestVoteResponse,
};

pub use crate::meridian_cluster_v010::communications_server::{
    Communications, CommunicationsServer,
};

use crate::Actions;

pub struct InternalClusterGrpcServer {
    receive_actions: Sender<Actions>,
    send_actions: Sender<Actions>,
}

impl InternalClusterGrpcServer {
    pub async fn init(
        receive_actions: Sender<Actions>,
        send_actions: Sender<Actions>,
    ) -> Result<InternalClusterGrpcServer, Box<dyn std::error::Error>> {
        Ok(InternalClusterGrpcServer {
            receive_actions,
            send_actions,
        })
    }
}

#[tonic::async_trait]
impl Communications for InternalClusterGrpcServer {
    async fn append_entries(
        &self,
        request: Request<AppendEntriesRequest>,
    ) -> Result<Response<AppendEntriesResponse>, Status> {
        self.send_actions
            .send(Actions::AppendEntriesRequest(request.into_inner()))
            .unwrap();

        let mut subscriber = self.receive_actions.subscribe();

        match subscriber.recv().await {
            Ok(Actions::AppendEntriesResponse(response)) => Ok(Response::new(response)),
            Err(error) => {
                println!("{:?}", error);
                let message = String::from("not good!");
                let status = Status::new(Code::NotFound, message);
                Err(status)
            }
            _ => {
                let message = String::from("not good!");
                let status = Status::new(Code::NotFound, message);
                Err(status)
            }
        }
    }

    async fn install_snapshot(
        &self,
        request: Request<InstallSnapshotRequest>,
    ) -> Result<Response<InstallSnapshotResponse>, Status> {
        let response = InstallSnapshotResponse { term: 1 };

        Ok(Response::new(response))
    }

    async fn request_vote(
        &self,
        request: Request<RequestVoteRequest>,
    ) -> Result<Response<RequestVoteResponse>, Status> {
        self.send_actions
            .send(Actions::RequestVoteRequest(request.into_inner()))
            .unwrap();

        let mut subscriber = self.receive_actions.subscribe();

        match subscriber.recv().await {
            Ok(Actions::RequestVoteResponse(response)) => Ok(Response::new(response)),
            Err(error) => {
                println!("{:?}", error);
                let message = String::from("not good!");
                let status = Status::new(Code::NotFound, message);
                Err(status)
            }
            _ => {
                let message = String::from("not good!");
                let status = Status::new(Code::NotFound, message);
                Err(status)
            }
        }
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     async fn test_channel() -> (Sender<u32>, Receiver<u32>) {
//         channel(1)
//     }
//
//     #[tokio::test(flavor = "multi_thread")]
//     async fn init() -> Result<(), Box<dyn std::error::Error>> {
//         let test_internal_cluster_grpc_server = InternalClusterGrpcServer::init().await;
//         assert!(test_internal_cluster_grpc_server.is_ok());
//         Ok(())
//     }
//
//     // #[tokio::test(flavor = "multi_thread")]
//     // async fn check_term_true() -> Result<(), Box<dyn std::error::Error>> {
//     //     let test_check_term = 6;
//     //     let test_internal_cluster_grpc_server = InternalClusterGrpcServer::init().await?;
//     //     assert!(
//     //         test_internal_cluster_grpc_server
//     //             .check_term(test_check_term)
//     //             .await
//     //     );
//     //     Ok(())
//     // }
//     //
//     // #[tokio::test(flavor = "multi_thread")]
//     // async fn check_term_false() -> Result<(), Box<dyn std::error::Error>> {
//     //     let test_check_term = 0;
//     //     let test_internal_cluster_grpc_server = InternalClusterGrpcServer::init().await?;
//     //     assert!(
//     //         !test_internal_cluster_grpc_server
//     //             .check_term(test_check_term)
//     //             .await
//     //     );
//     //     Ok(())
//     // }
//     //
//     // #[tokio::test(flavor = "multi_thread")]
//     // async fn check_candidate_id_none_true() -> Result<(), Box<dyn std::error::Error>> {
//     //     let test_voted_for = None;
//     //     let test_candidate_id = "some_test_candidate_id";
//     //     let test_internal_cluster_grpc_server = InternalClusterGrpcServer::init().await?;
//     //     assert!(
//     //         test_internal_cluster_grpc_server
//     //             .check_candidate_id(test_voted_for, test_candidate_id)
//     //             .await
//     //     );
//     //     Ok(())
//     // }
//     //
//     // #[tokio::test(flavor = "multi_thread")]
//     // async fn check_candidate_id_some_true() -> Result<(), Box<dyn std::error::Error>> {
//     //     let test_voted_for = Some("some_test_candidate_id");
//     //     let test_candidate_id = "some_test_candidate_id";
//     //     let test_internal_cluster_grpc_server = InternalClusterGrpcServer::init().await?;
//     //     assert!(
//     //         test_internal_cluster_grpc_server
//     //             .check_candidate_id(test_voted_for, test_candidate_id)
//     //             .await
//     //     );
//     //     Ok(())
//     // }
//     //
//     // #[tokio::test(flavor = "multi_thread")]
//     // async fn check_candidate_id_some_false() -> Result<(), Box<dyn std::error::Error>> {
//     //     let test_voted_for = Some("some_other_test_candidate_id");
//     //     let test_candidate_id = "some_test_candidate_id";
//     //     let test_internal_cluster_grpc_server = InternalClusterGrpcServer::init().await?;
//     //     assert!(
//     //         !test_internal_cluster_grpc_server
//     //             .check_candidate_id(test_voted_for, test_candidate_id)
//     //             .await
//     //     );
//     //     Ok(())
//     // }
//     //
//     // #[tokio::test(flavor = "multi_thread")]
//     // async fn check_candidate_log_true() -> Result<(), Box<dyn std::error::Error>> {
//     //     let test_log = 1;
//     //     let test_candidate_log = 1;
//     //     let test_internal_cluster_grpc_server = InternalClusterGrpcServer::init().await?;
//     //     assert!(
//     //         test_internal_cluster_grpc_server
//     //             .check_candidate_log(test_log, test_candidate_log)
//     //             .await
//     //     );
//     //     Ok(())
//     // }
//     //
//     // #[tokio::test(flavor = "multi_thread")]
//     // async fn check_candidate_log_false() -> Result<(), Box<dyn std::error::Error>> {
//     //     let test_log = 0;
//     //     let test_candidate_log = 1;
//     //     let test_internal_cluster_grpc_server = InternalClusterGrpcServer::init().await?;
//     //     assert!(
//     //         !test_internal_cluster_grpc_server
//     //             .check_candidate_log(test_log, test_candidate_log)
//     //             .await
//     //     );
//     //     Ok(())
//     // }
//
//     #[tokio::test(flavor = "multi_thread")]
//     async fn append_entries_true() -> Result<(), Box<dyn std::error::Error>> {
//         let test_internal_cluster_grpc_server =
//             InternalClusterGrpcServer::init().await?;
//         let test_request = Request::new(AppendEntriesRequest {
//             term: 1,
//             leader_id: String::from("test_leader_id"),
//             prev_log_index: 1,
//             prev_log_term: 1,
//             entries: Vec::with_capacity(0),
//             leader_commit: 1,
//         });
//         let test_response = test_internal_cluster_grpc_server
//             .append_entries(test_request)
//             .await?;
//         test_sender.send(1).await.unwrap();
//         assert_eq!(test_response.get_ref().term, 1);
//         assert_eq!(test_response.get_ref().success.as_str(), "true");
//         Ok(())
//     }
//
//     #[tokio::test(flavor = "multi_thread")]
//     async fn append_entries_false_term() -> Result<(), Box<dyn std::error::Error>> {
//         let test_internal_cluster_grpc_server = InternalClusterGrpcServer::init().await?;
//         let test_request = Request::new(AppendEntriesRequest {
//             term: 0,
//             leader_id: String::from("test_leader_id"),
//             prev_log_index: 0,
//             prev_log_term: 0,
//             entries: Vec::with_capacity(0),
//             leader_commit: 0,
//         });
//         let test_response = test_internal_cluster_grpc_server
//             .append_entries(test_request)
//             .await?;
//         assert_eq!(test_response.get_ref().term, 1);
//         assert_eq!(test_response.get_ref().success.as_str(), "false");
//         Ok(())
//     }
//
//     #[tokio::test(flavor = "multi_thread")]
//     async fn append_entries_false_log() -> Result<(), Box<dyn std::error::Error>> {
//         let test_internal_cluster_grpc_server = InternalClusterGrpcServer::init().await?;
//         let test_request = Request::new(AppendEntriesRequest {
//             term: 0,
//             leader_id: String::from("test_leader_id"),
//             prev_log_index: 0,
//             prev_log_term: 0,
//             entries: Vec::with_capacity(0),
//             leader_commit: 1,
//         });
//         let test_response = test_internal_cluster_grpc_server
//             .append_entries(test_request)
//             .await?;
//         assert_eq!(test_response.get_ref().term, 1);
//         assert_eq!(test_response.get_ref().success.as_str(), "false");
//         Ok(())
//     }
//
//     #[tokio::test(flavor = "multi_thread")]
//     async fn install_snapshot() -> Result<(), Box<dyn std::error::Error>> {
//         let test_internal_cluster_grpc_server = InternalClusterGrpcServer::init().await?;
//         let test_request = Request::new(InstallSnapshotRequest {
//             term: 1,
//             leader_id: String::from("some_leader_id"),
//             last_included_index: 1,
//             last_included_term: 1,
//             offset: Vec::with_capacity(0),
//             data: Vec::with_capacity(0),
//             done: String::from("true"),
//         });
//         let test_response = test_internal_cluster_grpc_server
//             .install_snapshot(test_request)
//             .await?;
//         assert_eq!(test_response.get_ref().term, 1);
//         Ok(())
//     }
//
//     #[tokio::test(flavor = "multi_thread")]
//     async fn request_vote_true() -> Result<(), Box<dyn std::error::Error>> {
//         let test_internal_cluster_grpc_server = InternalClusterGrpcServer::init().await?;
//         let test_request = Request::new(RequestVoteRequest {
//             term: 1,
//             candidate_id: String::from("some_candidate_id"),
//             last_log_index: 0,
//             last_log_term: 0,
//         });
//         let test_response = test_internal_cluster_grpc_server
//             .request_vote(test_request)
//             .await?;
//         assert_eq!(test_response.get_ref().term, 0);
//         assert_eq!(test_response.get_ref().vote_granted.as_str(), "true");
//         Ok(())
//     }
//
//     #[tokio::test(flavor = "multi_thread")]
//     async fn request_vote_false() -> Result<(), Box<dyn std::error::Error>> {
//         let test_internal_cluster_grpc_server = InternalClusterGrpcServer::init().await?;
//         let test_request = Request::new(RequestVoteRequest {
//             term: 1,
//             candidate_id: String::from("some_candidate_id"),
//             last_log_index: 1,
//             last_log_term: 1,
//         });
//         let test_response = test_internal_cluster_grpc_server
//             .request_vote(test_request)
//             .await?;
//         assert_eq!(test_response.get_ref().term, 0);
//         assert_eq!(test_response.get_ref().vote_granted.as_str(), "false");
//         Ok(())
//     }
// }
