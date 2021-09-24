use crate::node::Node;

pub(crate) mod channels;
pub(crate) mod external_client_grpc_server;
pub(crate) mod external_membership_grpc_server;
pub(crate) mod internal_cluster_grpc_client;
pub(crate) mod internal_cluster_grpc_server;
pub(crate) mod membership;
pub mod node;
pub(crate) mod server;
pub(crate) mod state;

pub(crate) mod meridian_client_v010 {
    include!("../../proto/meridian.client.v010.rs");
}

pub(crate) mod meridian_cluster_v010 {
    include!("../../proto/meridian.cluster.v010.rs");
}

pub(crate) mod meridian_membership_v010 {
    include!("../../proto/meridian.membership.v010.rs");
}

use crate::meridian_cluster_v010::{
    AppendEntriesRequest, AppendEntriesResponse, InstallSnapshotRequest, InstallSnapshotResponse,
    RequestVoteRequest, RequestVoteResponse,
};

use crate::meridian_membership_v010::{JoinClusterRequest, JoinClusterResponse};
