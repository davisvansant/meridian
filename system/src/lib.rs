pub mod external_client_grpc_server;
pub mod internal_cluster_grpc_client;
pub mod internal_cluster_grpc_server;
pub mod server;

pub(crate) mod meridian_client_v010 {
    include!("../../proto/meridian.client.v010.rs");
}

pub(crate) mod meridian_cluster_v010 {
    include!("../../proto/meridian.cluster.v010.rs");
}
