use tonic::{Request, Response, Status};

use crate::meridian_cluster_v010::{
    AppendEntriesRequest, AppendEntriesResponse, InstallSnapshotRequest, InstallSnapshotResponse,
    RequestVoteRequest, RequestVoteResponse,
};

pub use crate::meridian_cluster_v010::communications_server::{
    Communications, CommunicationsServer,
};

pub struct InternalClusterGrpcServer {}

impl InternalClusterGrpcServer {
    pub async fn init() -> Result<InternalClusterGrpcServer, Box<dyn std::error::Error>> {
        Ok(InternalClusterGrpcServer {})
    }
}

#[tonic::async_trait]
impl Communications for InternalClusterGrpcServer {
    async fn append_entries(
        &self,
        request: Request<AppendEntriesRequest>,
    ) -> Result<Response<AppendEntriesResponse>, Status> {
        let response = AppendEntriesResponse {
            term: 1,
            success: String::from("some_success"),
        };

        Ok(Response::new(response))
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
        let response = RequestVoteResponse {
            term: 1,
            vote_granted: String::from("true"),
        };

        Ok(Response::new(response))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test(flavor = "multi_thread")]
    async fn init() -> Result<(), Box<dyn std::error::Error>> {
        let test_internal_cluster_grpc_server = InternalClusterGrpcServer::init().await;
        assert!(test_internal_cluster_grpc_server.is_ok());
        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn append_entries() -> Result<(), Box<dyn std::error::Error>> {
        let test_internal_cluster_grpc_server = InternalClusterGrpcServer::init().await?;
        let test_request = Request::new(AppendEntriesRequest {
            term: 1,
            leader_id: String::from("test_leader_id"),
            prev_log_index: 1,
            prev_log_term: 1,
            entries: Vec::with_capacity(0),
            leader_commit: 1,
        });
        let test_response = test_internal_cluster_grpc_server
            .append_entries(test_request)
            .await?;
        assert_eq!(test_response.get_ref().term, 1);
        assert_eq!(test_response.get_ref().success.as_str(), "some_success");
        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn install_snapshot() -> Result<(), Box<dyn std::error::Error>> {
        let test_internal_cluster_grpc_server = InternalClusterGrpcServer::init().await?;
        let test_request = Request::new(InstallSnapshotRequest {
            term: 1,
            leader_id: String::from("some_leader_id"),
            last_included_index: 1,
            last_included_term: 1,
            offset: Vec::with_capacity(0),
            data: Vec::with_capacity(0),
            done: String::from("true"),
        });
        let test_response = test_internal_cluster_grpc_server
            .install_snapshot(test_request)
            .await?;
        assert_eq!(test_response.get_ref().term, 1);
        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn request_vote() -> Result<(), Box<dyn std::error::Error>> {
        let test_internal_cluster_grpc_server = InternalClusterGrpcServer::init().await?;
        let test_request = Request::new(RequestVoteRequest {
            term: 1,
            candidate_id: String::from("some_candidate_id"),
            last_log_index: 1,
            last_log_term: 1,
        });
        let test_response = test_internal_cluster_grpc_server
            .request_vote(test_request)
            .await?;
        assert_eq!(test_response.get_ref().term, 1);
        assert_eq!(test_response.get_ref().vote_granted.as_str(), "true");
        Ok(())
    }
}
