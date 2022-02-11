pub struct MembershipState {
    cluster_size: ClusterSize,
    server: Node,
    launch_nodes: Vec<SocketAddr>,
    members: HashMap<Uuid, Node>,
    receiver: MembershipReceiver,
}
