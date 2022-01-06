use crate::channel::{get_node, join_cluster, launch_nodes, peer_nodes, peer_status};
use crate::channel::{ClientSender, MembershipSender};

pub async fn run(
    client: &ClientSender,
    membership: &MembershipSender,
) -> Result<(), Box<dyn std::error::Error>> {
    let local_node = get_node(membership).await?;
    let local_address = local_node.build_address(local_node.membership_port).await;
    let launch_nodes = launch_nodes(membership).await?;

    for node in &launch_nodes {
        join_cluster(client, *node).await?;
    }

    let mut attempts = 0;
    let mut launch_status = Vec::with_capacity(2);

    while attempts <= 5 {
        let mut cluster_candidates = Vec::with_capacity(5);

        for node in &launch_nodes {
            let candidates = peer_nodes(client, *node).await?;

            for candidate in candidates {
                cluster_candidates.push(candidate);
            }

            cluster_candidates.dedup();
        }

        if let Ok(local_address) = cluster_candidates.binary_search(&local_address) {
            cluster_candidates.remove(local_address);
        }

        for candidate in cluster_candidates {
            join_cluster(client, candidate).await?;

            let peer_status = peer_status(client, candidate).await?;

            if peer_status == 2 {
                launch_status.push(1);
            }
        }

        if launch_status.len() == 2 {
            println!("preparing to launch...");

            break;
        }

        attempts += 1;
    }

    Ok(())
}
