use std::net::SocketAddr;
use std::str::FromStr;

use crate::channel::{join_cluster, launch_nodes, peer_nodes, peer_status};
use crate::channel::{ClientSender, MembershipSender};

pub async fn run(
    client: &ClientSender,
    membership: &MembershipSender, // peers: Vec<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let launch_nodes = launch_nodes(membership).await?;

    for node in launch_nodes {
        join_cluster(client, node).await?;
    }

    let mut attempts = 0;
    let mut launch_status = Vec::with_capacity(2);

    while attempts <= 5 {
        let cluster_candidates = peer_nodes(client).await?;

        for candidate in cluster_candidates {
            join_cluster(client, candidate).await?;

            let peer_status = peer_status(client).await?;

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
