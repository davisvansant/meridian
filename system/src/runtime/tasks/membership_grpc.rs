use crate::grpc::membership_server::{
    CommunicationsServer as MembershipServer, ExternalMembershipGrpcServer,
};
use crate::runtime::sync::membership_receive_task::ChannelMembershipReceiveAction;
use crate::runtime::sync::membership_send_grpc_task::ChannelMembershipSendGrpcAction;
use crate::runtime::tasks::JoinHandle;
use crate::runtime::tasks::Server;
use crate::runtime::tasks::SocketAddr;

pub async fn run_task(
    grpc_receive_membership_actions: ChannelMembershipSendGrpcAction,
    grpc_send_membership_actions: ChannelMembershipReceiveAction,
    socket_address: SocketAddr,
) -> Result<JoinHandle<()>, Box<dyn std::error::Error>> {
    let membership_grpc = ExternalMembershipGrpcServer::init(
        grpc_receive_membership_actions,
        grpc_send_membership_actions,
    )
    .await?;

    let grpc_service = MembershipServer::new(membership_grpc);
    let router = Server::builder()
        .add_service(grpc_service)
        .serve(socket_address);

    let handle = tokio::spawn(async move {
        println!("starting up membership...");
        if let Err(error) = router.await {
            println!(
                "something went wrong with the internal membership - {:?}",
                error,
            );
        }
    });

    Ok(handle)
}
