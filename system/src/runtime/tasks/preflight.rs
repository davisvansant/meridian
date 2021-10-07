use crate::grpc::membership_client::ExternalMembershipGrpcClient;
// use crate::node::Node;
use crate::runtime::launch::ChannelLaunch;
use crate::runtime::sync::membership_receive_task::ChannelMembershipReceiveTask;
use crate::runtime::sync::membership_receive_task::MembershipReceiveTask;
use crate::runtime::sync::membership_send_preflight_task::ChannelMembershipSendPreflightTask;
use crate::runtime::sync::membership_send_preflight_task::MembershipSendPreflightTask;
use crate::runtime::tasks::JoinHandle;
use crate::MembershipNode;

use tokio::time::{sleep, timeout, timeout_at, Duration, Instant};

pub async fn run_task(
    membership_receive_task: ChannelMembershipReceiveTask,
    preflight_receive_membership_task: ChannelMembershipSendPreflightTask,
    send_launch_action: ChannelLaunch,
    peers: Vec<String>,
) -> Result<JoinHandle<()>, Box<dyn std::error::Error>> {
    let mut receiver = preflight_receive_membership_task.subscribe();

    println!("waiting for init...");

    sleep(Duration::from_secs(10)).await;

    let handle = tokio::spawn(async move {
        membership_receive_task
            .send(MembershipReceiveTask::Node(1))
            .expect("error sending task...");

        let node = match receiver.recv().await {
            Ok(MembershipSendPreflightTask::NodeResponse(node)) => node,
            _ => panic!("expected node in response!"),
        };

        for member in &peers {
            println!("{:?}", member);

            let membership_node = MembershipNode {
                id: node.id.to_string(),
                address: node.address.to_string(),
                client_port: node.client_port.to_string(),
                cluster_port: node.cluster_port.to_string(),
                membership_port: node.membership_port.to_string(),
            };

            let mut endpoint = String::with_capacity(20);

            endpoint.push_str("http://");
            endpoint.push_str(member);
            endpoint.shrink_to_fit();

            println!("sending join request ... {:?}", &endpoint);

            let mut client = ExternalMembershipGrpcClient::init(endpoint).await.unwrap();
            let join_cluster_response = client.join_cluster(membership_node).await.unwrap();

            println!("join response - {:?}", join_cluster_response);

            membership_receive_task
                .send(MembershipReceiveTask::JoinCluster((
                    1,
                    join_cluster_response.into_inner(),
                )))
                .expect("error sending task...");

            sleep(Duration::from_secs(5)).await;

            let get_nodes_response = client.get_nodes().await.unwrap();

            println!("get nodes! - {:?}", &get_nodes_response);

            for cluster_node in get_nodes_response.into_inner().nodes {
                println!("sending response to peer - {:?}", node);

                let membership_node = MembershipNode {
                    id: node.id.to_string(),
                    address: node.address.to_string(),
                    client_port: node.client_port.to_string(),
                    cluster_port: node.cluster_port.to_string(),
                    membership_port: node.membership_port.to_string(),
                };

                let mut endpoint = String::with_capacity(20);

                endpoint.push_str("http://");
                endpoint.push_str(cluster_node.address.to_string().as_str());
                endpoint.push(':');
                endpoint.push_str(cluster_node.membership_port.to_string().as_str());
                endpoint.shrink_to_fit();

                println!("sending join request to known members... {:?}", &endpoint);

                let node_self = format!("http://{}:{}", node.address, node.membership_port);

                if endpoint == node_self {
                    println!("endpoint {:?} - self {:?}", &endpoint, &node_self);
                    println!("not sending request to self!");
                } else {
                    let mut client = ExternalMembershipGrpcClient::init(endpoint).await.unwrap();
                    let response = client.join_cluster(membership_node).await.unwrap();

                    println!("joining node - {:?}", response);
                }
            }
        }

        let mut attempts = 0;

        while attempts <= 5 {
            println!("starting attempts ! {:?}", &attempts);
            membership_receive_task
                .send(MembershipReceiveTask::Members(1))
                .expect("error sending task");

            let members = match receiver.recv().await {
                Ok(MembershipSendPreflightTask::MembersResponse(members)) => members,
                _ => panic!("expected members in response!"),
            };

            println!("members - {:?}", &members.len());

            let mut status = Vec::with_capacity(2);

            for member in &members {
                println!("members - {:?}", member);

                let mut endpoint = String::with_capacity(20);

                endpoint.push_str("http://");
                endpoint.push_str(member.address.to_string().as_str());
                endpoint.push(':');
                endpoint.push_str(member.membership_port.to_string().as_str());
                endpoint.shrink_to_fit();

                println!("sending status to known members... {:?}", &endpoint);

                if endpoint == format!("http://{}:{}", node.address, node.membership_port) {
                    println!("not sending request to self!");
                } else {
                    let mut client = ExternalMembershipGrpcClient::init(endpoint).await.unwrap();
                    let response = client.get_node_status().await.unwrap();

                    println!("join response - {:?}", response);

                    if response.into_inner().status.as_str() == "2" {
                        status.push(1);
                    } else {
                        println!("node is not yet ready!");
                    }
                }
            }

            if status.len() == 2 {
                println!("preparing to launch...");

                sleep(Duration::from_secs(10)).await;

                if send_launch_action.send(()).is_ok() {
                    println!("sending launch action");
                }

                break;
            }

            attempts += 1;

            sleep(Duration::from_secs(5)).await;
        }
    });

    Ok(handle)
}
