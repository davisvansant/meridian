// use crate::grpc::membership_client::ExternalMembershipGrpcClient;
use crate::node::Node;
// use crate::runtime::launch::ChannelLaunch;
// use crate::runtime::sync::membership_receive_task::ChannelMembershipReceiveTask;
// use crate::runtime::sync::membership_receive_task::MembershipReceiveTask;
// use crate::runtime::sync::membership_send_preflight_task::ChannelMembershipSendPreflightTask;
// use crate::runtime::sync::membership_send_preflight_task::MembershipSendPreflightTask;
use crate::runtime::tasks::JoinHandle;
// use crate::MembershipNode;
use std::str::FromStr;
use tokio::time::{sleep, timeout, timeout_at, Duration, Instant};
use tonic::transport::Endpoint;

// use crate::rpc::membership::MembershipNode as RpcMembershipNode;
use crate::rpc::{Client, Data, Interface};
// use crate::rpc::membership::MembershipNode as RpcMembershipNode;

use flexbuffers::{Builder, BuilderOptions, FlexbufferSerializer, MapBuilder};

use flexbuffers::Pushable;

use crate::channel::MembershipSender;

// use crate::channel::{get_node, join_cluster};
use crate::channel::get_node;

pub async fn run_task(
    // membership_receive_task: ChannelMembershipReceiveTask,
    // preflight_receive_membership_task: ChannelMembershipSendPreflightTask,
    // send_launch_action: ChannelLaunch,
    peers: Vec<String>,
    membership_sender: MembershipSender,
) -> Result<JoinHandle<()>, Box<dyn std::error::Error>> {
    println!("waiting for init...");

    sleep(Duration::from_secs(10)).await;

    let handle = tokio::spawn(async move {
        if let Err(error) = preflight(
            peers,
            // membership_receive_task.clone(),
            // preflight_receive_membership_task.clone(),
            membership_sender,
        )
        .await
        {
            println!("Prelight error! {:?}", error);
        };

        if let Err(error) = attempts(
            // membership_receive_task.clone(),
            // preflight_receive_membership_task.clone(),
            // send_launch_action,
        )
        .await
        {
            println!("Preflight Attemps error! {:?}", error);
        };
    });

    Ok(handle)
}

async fn preflight(
    peers: Vec<String>,
    // sender_channel: ChannelMembershipReceiveTask,
    // receiver_channel: ChannelMembershipSendPreflightTask,
    membership_sender: MembershipSender,
) -> Result<(), Box<dyn std::error::Error>> {
    // send_membership_node_task(sender_channel.clone()).await?;
    // let node = receive_membership_node(receiver_channel).await?;
    let node = get_node(&membership_sender).await?;

    for member in &peers {
        println!("{:?}", member);

        // let membership_node = build_membership_node(node).await?;
        let (address, port) = member.split_once(":").unwrap();
        let endpoint = build_endpoint(address.to_string(), port.to_string()).await?;

        // let rpc_client = Client::init(Interface::Membership).await?;
        // let join_cluster_request = Data::JoinClusterRequest(node).build().await?;
        // let join_cluster_response = rpc_client.transmit(&join_cluster_request).await?;

        // let mut test_flexbuffers_builder = Builder::new(BuilderOptions::SHARE_NONE);

        // join_cluster_response.push_to_builder(&mut test_flexbuffers_builder);

        // let test_flexbuffer_root = flexbuffers::Reader::get_root(test_flexbuffers_builder.view())?;

        // let test_flexbuffers_root_details = test_flexbuffer_root.as_map().idx("details").as_map();

        // // println!("{:?}", test_flexbuffers_root_details);
        // for value in test_flexbuffers_root_details.iter_values() {
        //     println!("some value here - {:?}", value.as_str());
        // }

        // rpc_client.join_cluster().await?;

        // send_membership_join_cluster_task(
        //     sender_channel.clone(),
        //     join_cluster_response.into_inner(),
        // )
        // .await?;

        sleep(Duration::from_secs(5)).await;

        // let another_rpc_client = Client::init(Interface::Membership).await?;

        // let connected_nodes = rpc_client.get_connected().await?;

        // for connected_node in connected_nodes.iter() {
        //     let endpoint = build_endpoint(
        //         connected_node.address.to_owned(),
        //         connected_node.membership_port.to_owned(),
        //     )
        //     .await?;

        //     let self_endpoint =
        //         build_endpoint(node.address.to_string(), node.membership_port.to_string()).await?;

        //     if endpoint.uri() == self_endpoint.uri() {
        //         println!("endpoint {:?} - self {:?}", &endpoint, &self_endpoint);
        //         println!("not sending request to self!");
        //     } else {
        //         // let mut client = ExternalMembershipGrpcClient::init(endpoint).await.unwrap();
        //         // let response = client.join_cluster(membership_node).await.unwrap();
        //         rpc_client.join_cluster().await?;

        //         // println!("joining node - {:?}", response);
        //     }
        // }
        // let connected_nodes = another_rpc_client.get_connected().await?;

        // let client = Client::init(Interface::Membership).await?;

        // let get_connected_request = Data::ConnectedRequest.build().await?;
        // let get_connected_response = rpc_client.transmit(&get_connected_request).await?;

        // // println!("get connected response - {:?}", String::from_utf8(get_connected_response)?);

        // let mut another_test_flexbuffers_builder = Builder::new(BuilderOptions::SHARE_NONE);

        // get_connected_response.push_to_builder(&mut another_test_flexbuffers_builder);

        // let another_test_flexbuffer_root =
        //     flexbuffers::Reader::get_root(another_test_flexbuffers_builder.view())?;

        // let test_flexbuffers_root_details = another_test_flexbuffer_root.as_vector();

        // for value in test_flexbuffers_root_details.iter() {
        //     println!("some value here - {:?}", value.as_str());
        // }

        // let get_nodes_response = client.get_nodes().await?;

        // for cluster_node in get_nodes_response.into_inner().nodes {
        //     let membership_node = build_membership_node(node).await?;
        //     let endpoint =
        //         build_endpoint(cluster_node.address, cluster_node.membership_port).await?;

        //     let self_endpoint =
        //         build_endpoint(node.address.to_string(), node.membership_port.to_string()).await?;

        //     if endpoint.uri() == self_endpoint.uri() {
        //         println!("endpoint {:?} - self {:?}", &endpoint, &self_endpoint);
        //         println!("not sending request to self!");
        //     } else {
        //         let mut client = ExternalMembershipGrpcClient::init(endpoint).await.unwrap();
        //         let response = client.join_cluster(membership_node).await.unwrap();

        //         println!("joining node - {:?}", response);
        //     }
        // }
    }

    Ok(())
}

async fn attempts(// sender_channel: ChannelMembershipReceiveTask,
    // receiver_channel: ChannelMembershipSendPreflightTask,
    // send_launch_action: ChannelLaunch,
) -> Result<(), Box<dyn std::error::Error>> {
    // let mut attempts = 0;

    // while attempts <= 5 {
    //     println!("starting attempts ! {:?}", &attempts);

    //     send_membership_members_task(sender_channel.clone()).await?;
    //     let members = receive_membership_members(receiver_channel.clone()).await?;
    //     let mut status = Vec::with_capacity(2);

    //     for member in &members {
    //         println!("members - {:?}", member);

    //         let endpoint = build_endpoint(
    //             member.address.to_string(),
    //             member.membership_port.to_string(),
    //         )
    //         .await?;

    //         println!("sending status to known members... {:?}", &endpoint);

    //         let mut client = ExternalMembershipGrpcClient::init(endpoint).await?;
    //         let response = client.get_node_status().await?;

    //         println!("join response - {:?}", response);

    //         if response.into_inner().status.as_str() == "2" {
    //             status.push(1);
    //         } else {
    //             println!("node is not yet ready!");
    //         }
    //     }

    //     if status.len() == 2 {
    //         println!("preparing to launch...");

    //         sleep(Duration::from_secs(10)).await;

    //         if send_launch_action.send(()).is_ok() {
    //             println!("sending launch action");
    //         }

    //         break;
    //     }

    //     attempts += 1;

    //     sleep(Duration::from_secs(5)).await;
    // }

    // if send_launch_action.send(()).is_ok() {
    //     println!("sending launch action");
    // }

    Ok(())
}

pub async fn build_endpoint(
    address: String,
    port: String,
) -> Result<Endpoint, Box<dyn std::error::Error>> {
    let mut endpoint = String::with_capacity(20);
    let scheme = "http://";

    endpoint.push_str(scheme);
    endpoint.push_str(&address);
    endpoint.push(':');
    endpoint.push_str(&port);
    endpoint.shrink_to_fit();

    Ok(Endpoint::from_str(&endpoint)?)
}

// async fn build_membership_node(node: Node) -> Result<MembershipNode, Box<dyn std::error::Error>> {
//     let membership_node = MembershipNode {
//         id: node.id.to_string(),
//         address: node.address.to_string(),
//         client_port: node.client_port.to_string(),
//         cluster_port: node.cluster_port.to_string(),
//         membership_port: node.membership_port.to_string(),
//     };

//     Ok(membership_node)
// }

// async fn send_membership_node_task(
//     sender: ChannelMembershipReceiveTask,
// ) -> Result<(), Box<dyn std::error::Error>> {
//     sender.send(MembershipReceiveTask::Node(1))?;

//     Ok(())
// }

// async fn receive_membership_node(
//     channel: ChannelMembershipSendPreflightTask,
// ) -> Result<Node, Box<dyn std::error::Error>> {
//     let mut receiver = channel.subscribe();

//     let node = receiver.recv().await?;
//     if let MembershipSendPreflightTask::NodeResponse(node) = node {
//         Ok(node)
//     } else {
//         panic!("received unexpected results")
//     }
// }

// async fn send_membership_members_task(
//     channel: ChannelMembershipReceiveTask,
// ) -> Result<(), Box<dyn std::error::Error>> {
//     channel.send(MembershipReceiveTask::Members(1))?;

//     Ok(())
// }

// async fn receive_membership_members(
//     channel: ChannelMembershipSendPreflightTask,
// ) -> Result<Vec<Node>, Box<dyn std::error::Error>> {
//     let mut receiver = channel.subscribe();

//     let members = receiver.recv().await?;

//     if let MembershipSendPreflightTask::MembersResponse(nodes) = members {
//         Ok(nodes)
//     } else {
//         panic!("received unexpected result!")
//     }
// }

// async fn send_membership_join_cluster_task(
//     channel: ChannelMembershipReceiveTask,
//     node: MembershipNode,
// ) -> Result<(), Box<dyn std::error::Error>> {
//     channel.send(MembershipReceiveTask::JoinCluster((1, node)))?;

//     Ok(())
// }
