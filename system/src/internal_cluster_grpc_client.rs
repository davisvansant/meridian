use tonic::transport::Channel;
use tonic::{Request, Response, Status};

use crate::meridian_cluster_v010::{
    AppendEntriesRequest, AppendEntriesResponse, InstallSnapshotRequest, InstallSnapshotResponse,
    RequestVoteRequest, RequestVoteResponse,
};

pub use crate::meridian_cluster_v010::communications_client::CommunicationsClient;

pub struct InternalClusterGrpcClient {
    transport: CommunicationsClient<Channel>,
}

impl InternalClusterGrpcClient {
    pub async fn init(
        endpoint: &'static str,
    ) -> Result<InternalClusterGrpcClient, Box<dyn std::error::Error>> {
        let transport = CommunicationsClient::connect(endpoint).await?;

        Ok(InternalClusterGrpcClient { transport })
    }

    pub async fn append_entries(&mut self) -> Result<Response<AppendEntriesResponse>, Status> {
        let mut transport = self.transport.clone();
        let request = Request::new(AppendEntriesRequest {
            term: 1,
            leader_id: String::from("some_leader_id"),
            prev_log_index: 1,
            prev_log_term: 1,
            entries: Vec::with_capacity(10),
            leader_commit: 1,
        });
        let response = transport.append_entries(request).await?;

        Ok(response)
    }

    pub async fn install_snapshot(&mut self) -> Result<Response<InstallSnapshotResponse>, Status> {
        let mut transport = self.transport.clone();
        let request = Request::new(InstallSnapshotRequest {
            term: 1,
            leader_id: String::from("some_leader_id"),
            last_included_index: 1,
            last_included_term: 1,
            offset: Vec::with_capacity(10),
            data: Vec::with_capacity(10),
            done: String::from("done"),
        });
        let response = transport.install_snapshot(request).await?;

        Ok(response)
    }

    pub async fn request_vote(&mut self) -> Result<Response<RequestVoteResponse>, Status> {
        let mut transport = self.transport.clone();
        let request = Request::new(RequestVoteRequest {
            term: 1,
            candidate_id: String::from("some_candidate_id"),
            last_log_index: 1,
            last_log_term: 1,
        });
        let response = transport.request_vote(request).await?;

        Ok(response)
    }
}
