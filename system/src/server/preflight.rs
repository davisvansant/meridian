use std::net::SocketAddr;
use std::str::FromStr;

use crate::channel::ClientSender;
use crate::channel::{join_cluster, peer_nodes, peer_status};

pub async fn run(
    client: &ClientSender,
    peers: Vec<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    for peer_address in peers {
        let socket_address = SocketAddr::from_str(&peer_address)?;

        join_cluster(client, socket_address).await?;
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
