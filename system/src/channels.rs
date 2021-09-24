use tokio::sync::broadcast::channel;
use tokio::sync::broadcast::Sender;

use crate::AppendEntriesRequest;
use crate::AppendEntriesResponse;
use crate::JoinClusterRequest;
use crate::JoinClusterResponse;
use crate::Node;
use crate::RequestVoteRequest;
use crate::RequestVoteResponse;

pub type ChannelStateReceiveAction = Sender<StateReceiveAction>;
pub type ChannelStateSendGrpcAction = Sender<StateSendGrpcAction>;
pub type ChannelStateSendServerAction = Sender<StateSendServerAction>;
pub type ChannelMembershipReceiveAction = Sender<MembershipReceiveAction>;
pub type ChannelMembershipSendGrpcAction = Sender<MembershipSendGrpcAction>;
pub type ChannelMembershipSendServerAction = Sender<MembershipSendServerAction>;

#[derive(Clone, Debug)]
pub enum StateReceiveAction {
    AppendEntriesRequest(AppendEntriesRequest),
    RequestVoteRequest(RequestVoteRequest),
    Candidate(String),
    Leader(String),
}

pub async fn channel_state_receive_action() -> (
    ChannelStateReceiveAction,
    ChannelStateReceiveAction,
    ChannelStateReceiveAction,
) {
    let (state_send_handle, _) = channel(64);
    let grpc_send_actions = state_send_handle.clone();
    let server_send_actions = state_send_handle.clone();

    (state_send_handle, grpc_send_actions, server_send_actions)
}

#[derive(Clone, Debug)]
pub enum StateSendGrpcAction {
    AppendEntriesResponse(AppendEntriesResponse),
    RequestVoteResponse(RequestVoteResponse),
}

pub async fn channel_state_send_grpc_action(
) -> (ChannelStateSendGrpcAction, ChannelStateSendGrpcAction) {
    let (state_receive_grpc_actions, _) = channel(64);
    let state_send_grpc_actions = state_receive_grpc_actions.clone();

    (state_receive_grpc_actions, state_send_grpc_actions)
}

#[derive(Clone, Debug)]
pub enum StateSendServerAction {
    AppendEntriesRequest(AppendEntriesRequest),
    RequestVoteRequest(RequestVoteRequest),
    Follower,
}

pub async fn channel_state_send_server_action(
) -> (ChannelStateSendServerAction, ChannelStateSendServerAction) {
    let (state_receive_server_actions, _) = channel(64);
    let state_send_server_actions = state_receive_server_actions.clone();

    (state_receive_server_actions, state_send_server_actions)
}

#[derive(Clone, Debug)]
pub enum MembershipReceiveAction {
    JoinClusterRequest(JoinClusterRequest),
    Node,
    Members,
}

pub async fn channel_membership_receive_action() -> (
    ChannelMembershipReceiveAction,
    ChannelMembershipReceiveAction,
    ChannelMembershipReceiveAction,
) {
    let (membership_receive_action, _) = channel(64);
    let membership_grpc_send_membership_action = membership_receive_action.clone();
    let server_send_membership_action = membership_receive_action.clone();

    (
        membership_receive_action,
        membership_grpc_send_membership_action,
        server_send_membership_action,
    )
}

#[derive(Clone, Debug)]
pub enum MembershipSendGrpcAction {
    JoinClusterResponse(JoinClusterResponse),
}

pub async fn channel_membership_send_grpc_action() -> (
    ChannelMembershipSendGrpcAction,
    ChannelMembershipSendGrpcAction,
) {
    let (membership_send_membership_grpc_action, _) = channel(64);
    let membership_grpc_receive_membership_action = membership_send_membership_grpc_action.clone();

    (
        membership_send_membership_grpc_action,
        membership_grpc_receive_membership_action,
    )
}

#[derive(Clone, Debug)]
pub enum MembershipSendServerAction {
    NodeResponse(Node),
    MembersResponse(Vec<Node>),
}

pub async fn channel_membership_send_server_action() -> (
    ChannelMembershipSendServerAction,
    ChannelMembershipSendServerAction,
) {
    let (membership_send_server_action, _) = channel(64);
    let server_receive_membership_action = membership_send_server_action.clone();

    (
        membership_send_server_action,
        server_receive_membership_action,
    )
}
