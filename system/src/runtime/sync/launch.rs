use crate::runtime::sync::channel;
use crate::runtime::sync::Sender;

pub type ChannelLaunch = Sender<()>;

pub async fn build_channel() -> (ChannelLaunch, ChannelLaunch, ChannelLaunch) {
    let (tx, _) = channel(8);
    let client_grpc_receiver = tx.clone();
    let cluster_grpc_receiver = tx.clone();

    (tx, client_grpc_receiver, cluster_grpc_receiver)
}
