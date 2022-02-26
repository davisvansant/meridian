// use tokio::time::{sleep, timeout, timeout_at, Duration, Instant};
// use tokio::time::{sleep, Duration};

// use crate::channel::{get_node, join_cluster, launch_nodes, peer_nodes};
// use crate::channel::{ClientSender, MembershipSender};

use crate::channel::static_join;
use crate::channel::MembershipSender;

pub async fn run(
    // client: &ClientSender,
    membership: &MembershipSender,
) -> Result<(), Box<dyn std::error::Error>> {
    // let local_node = get_node(membership).await?;
    // let local_address = local_node.build_address(local_node.cluster_port).await;

    // let mut cluster_candidates = Vec::with_capacity(5);

    // let launch_nodes = launch_nodes(membership).await?;

    // for node in &launch_nodes {
    //     join_cluster(client, *node).await?;

    //     let peers = peer_nodes(client, *node).await?;

    //     for peer in peers {
    //         cluster_candidates.push(peer);
    //     }
    // }

    // let mut attempts = 0;

    // while attempts <= 3 {
    //     let mut additional_candidates = Vec::with_capacity(5);

    //     for candidate in &mut cluster_candidates {
    //         let peer_nodes = peer_nodes(client, *candidate).await?;

    //         for peer in peer_nodes {
    //             println!("peer -> {:?}", &peer);
    //             additional_candidates.push(peer);
    //         }
    //     }

    //     for candidate in additional_candidates {
    //         cluster_candidates.push(candidate)
    //     }

    //     attempts += 1;
    // }

    // cluster_candidates.sort();
    // cluster_candidates.dedup();

    // if let Ok(local_address) = cluster_candidates.binary_search(&local_address) {
    //     println!("removing local address from candidates...");
    //     cluster_candidates.remove(local_address);
    // }

    // cluster_candidates.sort();
    // cluster_candidates.dedup();

    // for candidate in &cluster_candidates {
    //     join_cluster(client, *candidate).await?;

    //     let peer_nodes = peer_nodes(client, *candidate).await?;

    //     for peer in peer_nodes {
    //         join_cluster(client, peer).await?;
    //     }
    // }

    // sleep(Duration::from_secs(5)).await;

    // println!("launching...");
    let (active, expected) = static_join(membership).await?;

    if active == expected {
        Ok(())
    } else {
        let error = format!("static join -> expected {} | active {}", &expected, &active);

        Err(Box::from(error))
    }
}
