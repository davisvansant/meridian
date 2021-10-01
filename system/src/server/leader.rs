use crate::grpc::cluster_client::InternalClusterGrpcClient;
use crate::meridian_cluster_v010::AppendEntriesRequest;

pub struct Leader {}

impl Leader {
    pub async fn init() -> Result<Leader, Box<dyn std::error::Error>> {
        Ok(Leader {})
    }

    pub async fn send_heartbeat(
        &self,
        request: AppendEntriesRequest,
        address: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!("sending heartbeat...");

        let mut transport = InternalClusterGrpcClient::init(address).await?;
        let result = transport.append_entries(request).await?;

        println!("{:?}", result);

        Ok(())
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     #[tokio::test(flavor = "multi_thread")]
//     async fn init() -> Result<(), Box<dyn std::error::Error>> {
//         let test_leader = Leader::init().await?;
//         assert_eq!(test_leader.volatile_state.next_index, 0);
//         assert_eq!(test_leader.volatile_state.match_index, 0);
//         Ok(())
//     }
//
//     #[tokio::test(flavor = "multi_thread")]
//     async fn send_heartbeat() -> Result<(), Box<dyn std::error::Error>> {
//         let test_server = test_server().await?;
//         let test_leader_volatile_state = VolatileState::init().await?;
//         let test_request = test_server
//             .build_append_entires_request(&test_leader_volatile_state)
//             .await?;
//         let test_leader = Leader::init().await?;
//         assert!(test_leader.send_heartbeat(test_request).await.is_ok());
//         Ok(())
//     }
//
//     #[tokio::test(flavor = "multi_thread")]
//     async fn build_append_entires_request() -> Result<(), Box<dyn std::error::Error>> {
//         let test_server = test_server().await?;
//         let test_leader_volatile_state = VolatileState::init().await?;
//         let test_request = test_server
//             .build_append_entires_request(&test_leader_volatile_state)
//             .await?;
//         assert_eq!(test_request.term, 0);
//         assert_eq!(test_request.leader_id.as_str(), "some_leader_id");
//         assert_eq!(test_request.prev_log_index, 0);
//         assert_eq!(test_request.prev_log_term, 0);
//         assert_eq!(test_request.entries.len(), 0);
//         assert_eq!(test_request.leader_commit, 0);
//         Ok(())
//     }
// }
