pub mod external_client_grpc_server;
pub mod internal_cluster_grpc_client;
pub mod internal_cluster_grpc_server;
pub mod server;
pub mod state;

pub(crate) mod meridian_client_v010 {
    include!("../../proto/meridian.client.v010.rs");
}

pub(crate) mod meridian_cluster_v010 {
    include!("../../proto/meridian.cluster.v010.rs");
}

use crate::meridian_cluster_v010::{
    AppendEntriesRequest, AppendEntriesResponse, InstallSnapshotRequest, InstallSnapshotResponse,
    RequestVoteRequest, RequestVoteResponse,
};

#[derive(Clone, Debug, PartialEq)]
pub enum Actions {
    AppendEntriesRequest(AppendEntriesRequest),
    AppendEntriesResponse(AppendEntriesResponse),
    RequestVoteRequest(RequestVoteRequest),
    RequestVoteResponse(RequestVoteResponse),
    Candidate(String),
    Follower,
    Leader,
}
