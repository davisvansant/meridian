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

    async fn check_term(&self, term: u32) -> bool {
        let current_term = 0;
        term > current_term
    }

    async fn check_candidate_id(&self, voted_for: Option<&str>, candidate_id: &str) -> bool {
        voted_for == None || voted_for == Some(candidate_id)
    }

    async fn check_candidate_log(&self, log: u32, candidate_log: u32) -> bool {
        log >= candidate_log
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
        let self_term = 0;
        let self_log = 0;
        // let self_voted_for = Some("some_candidate_id");
        let self_voted_for = None;

        let true_response = RequestVoteResponse {
            term: self_term,
            vote_granted: String::from("true"),
        };

        let false_response = RequestVoteResponse {
            term: self_term,
            vote_granted: String::from("false"),
        };

        match self.check_term(request.get_ref().term).await {
            false => Ok(Response::new(false_response)),
            true => {
                match self
                    .check_candidate_id(self_voted_for, request.get_ref().candidate_id.as_str())
                    .await
                    && self
                        .check_candidate_log(self_log, request.get_ref().last_log_term)
                        .await
                {
                    true => Ok(Response::new(true_response)),
                    false => Ok(Response::new(false_response)),
                }
            }
        }
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
    async fn check_term_true() -> Result<(), Box<dyn std::error::Error>> {
        let test_check_term = 6;
        let test_internal_cluster_grpc_server = InternalClusterGrpcServer::init().await?;
        assert!(
            test_internal_cluster_grpc_server
                .check_term(test_check_term)
                .await
        );
        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn check_term_false() -> Result<(), Box<dyn std::error::Error>> {
        let test_check_term = 0;
        let test_internal_cluster_grpc_server = InternalClusterGrpcServer::init().await?;
        assert!(
            !test_internal_cluster_grpc_server
                .check_term(test_check_term)
                .await
        );
        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn check_candidate_id_none_true() -> Result<(), Box<dyn std::error::Error>> {
        let test_voted_for = None;
        let test_candidate_id = "some_test_candidate_id";
        let test_internal_cluster_grpc_server = InternalClusterGrpcServer::init().await?;
        assert!(
            test_internal_cluster_grpc_server
                .check_candidate_id(test_voted_for, test_candidate_id)
                .await
        );
        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn check_candidate_id_some_true() -> Result<(), Box<dyn std::error::Error>> {
        let test_voted_for = Some("some_test_candidate_id");
        let test_candidate_id = "some_test_candidate_id";
        let test_internal_cluster_grpc_server = InternalClusterGrpcServer::init().await?;
        assert!(
            test_internal_cluster_grpc_server
                .check_candidate_id(test_voted_for, test_candidate_id)
                .await
        );
        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn check_candidate_id_some_false() -> Result<(), Box<dyn std::error::Error>> {
        let test_voted_for = Some("some_other_test_candidate_id");
        let test_candidate_id = "some_test_candidate_id";
        let test_internal_cluster_grpc_server = InternalClusterGrpcServer::init().await?;
        assert!(
            !test_internal_cluster_grpc_server
                .check_candidate_id(test_voted_for, test_candidate_id)
                .await
        );
        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn check_candidate_log_true() -> Result<(), Box<dyn std::error::Error>> {
        let test_log = 1;
        let test_candidate_log = 1;
        let test_internal_cluster_grpc_server = InternalClusterGrpcServer::init().await?;
        assert!(
            test_internal_cluster_grpc_server
                .check_candidate_log(test_log, test_candidate_log)
                .await
        );
        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn check_candidate_log_false() -> Result<(), Box<dyn std::error::Error>> {
        let test_log = 0;
        let test_candidate_log = 1;
        let test_internal_cluster_grpc_server = InternalClusterGrpcServer::init().await?;
        assert!(
            !test_internal_cluster_grpc_server
                .check_candidate_log(test_log, test_candidate_log)
                .await
        );
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
    async fn request_vote_true() -> Result<(), Box<dyn std::error::Error>> {
        let test_internal_cluster_grpc_server = InternalClusterGrpcServer::init().await?;
        let test_request = Request::new(RequestVoteRequest {
            term: 1,
            candidate_id: String::from("some_candidate_id"),
            last_log_index: 0,
            last_log_term: 0,
        });
        let test_response = test_internal_cluster_grpc_server
            .request_vote(test_request)
            .await?;
        assert_eq!(test_response.get_ref().term, 0);
        assert_eq!(test_response.get_ref().vote_granted.as_str(), "true");
        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn request_vote_false() -> Result<(), Box<dyn std::error::Error>> {
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
        assert_eq!(test_response.get_ref().term, 0);
        assert_eq!(test_response.get_ref().vote_granted.as_str(), "false");
        Ok(())
    }
}
