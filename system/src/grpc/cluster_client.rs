use crate::grpc::{Channel, Request, Response, Status};
pub use crate::meridian_cluster_v010::communications_client::CommunicationsClient;
use crate::meridian_cluster_v010::{
    AppendEntriesRequest, AppendEntriesResponse, InstallSnapshotRequest, InstallSnapshotResponse,
    RequestVoteRequest, RequestVoteResponse,
};
use tonic::transport::Endpoint;

pub struct InternalClusterGrpcClient {
    transport: CommunicationsClient<Channel>,
}

impl InternalClusterGrpcClient {
    pub async fn init(
        endpoint: Endpoint,
    ) -> Result<InternalClusterGrpcClient, Box<dyn std::error::Error>> {
        let transport = CommunicationsClient::connect(endpoint).await?;

        Ok(InternalClusterGrpcClient { transport })
    }

    pub async fn append_entries(
        &mut self,
        request: AppendEntriesRequest,
    ) -> Result<Response<AppendEntriesResponse>, Status> {
        let mut transport = self.transport.clone();
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

    pub async fn request_vote(
        &mut self,
        request: RequestVoteRequest,
    ) -> Result<Response<RequestVoteResponse>, Status> {
        let mut transport = self.transport.clone();
        let response = transport.request_vote(request).await?;

        Ok(response)
    }
}
