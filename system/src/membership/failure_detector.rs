use std::net::IpAddr;
use std::str::FromStr;

use crate::node::Node;

use tokio::time::{sleep, timeout, Duration};

use crate::channel::membership::failure_detector::{
    FailureDetectorProtocolReceiver, PingTarget, PingTargetSender,
};
use crate::channel::membership::list::{ListRequest, ListSender};
use crate::channel::membership::sender::{Dissemination, DisseminationSender};
use crate::channel::transition::ShutdownSender;
use crate::membership::Message;
use crate::{error, info, warn};

pub struct FailureDectector {
    protocol_period: Duration,
    time_out: Duration,
    list_sender: ListSender,
    dissemination: DisseminationSender,
    ping_target_channel: PingTargetSender,
    enter_state: FailureDetectorProtocolReceiver,
    shutdown: ShutdownSender,
}

impl FailureDectector {
    pub async fn init(
        list_sender: ListSender,
        dissemination: DisseminationSender,
        ping_target_channel: PingTargetSender,
        enter_state: FailureDetectorProtocolReceiver,
        shutdown: ShutdownSender,
    ) -> FailureDectector {
        let protocol_period = Duration::from_secs(5);
        let time_out = Duration::from_secs(2);

        info!("initialized!");

        FailureDectector {
            protocol_period,
            time_out,
            list_sender,
            dissemination,
            ping_target_channel,
            enter_state,
            shutdown,
        }
    }

    pub async fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut shutdown = self.shutdown.subscribe();

        loop {
            tokio::select! {
                biased;
                _ = shutdown.recv() => {
                    info!("shutting down!");

                    break;
                }

                Some(run) = self.enter_state.recv() => {
                    info!("failure detector -> {:?}", run);
                    info!("Protocol Period -> {:?}", self.protocol_period);
                    info!("Time Out -> {:?}", self.time_out);

                    self.enter_state.close();

                    loop {
                        self.protocol_period().await?;
                    }
                }
            }
        }

        Ok(())
    }

    async fn protocol_period(&self) -> Result<(), Box<dyn std::error::Error>> {
        sleep(self.protocol_period).await;

        let direct_list = ListRequest::get_alive(&self.list_sender).await?;
        let mut suspected = Vec::with_capacity(1);

        for ping_target in direct_list {
            if let Err(error) = self.send_direct(ping_target).await {
                error!("ping target direct error -> {:?}", error);

                suspected.push(ping_target);

                break;
            }
        }

        if !suspected.is_empty() {
            let indirect_list = ListRequest::get_alive(&self.list_sender).await?;

            for ping_req_target in indirect_list {
                if let Err(error) = self.send_indirect(ping_req_target, suspected[0]).await {
                    error!("indirect probe failure -> {:?}", error);
                    error!("confirmed member as faulty -> {:?}", &suspected[0]);

                    break;
                }
            }
        }

        let suspected_list = ListRequest::get_suspected(&self.list_sender).await?;

        for member in suspected_list {
            self.confirmed(&member).await?;
        }

        Ok(())
    }

    async fn send_direct(&self, member: Node) -> Result<(), Box<dyn std::error::Error>> {
        let mut ping_target_ack = self.ping_target_channel.subscribe();

        let node = ListRequest::get_node(&self.list_sender).await?;
        let local_alive_list = ListRequest::get_alive(&self.list_sender).await?;
        let local_suspected_list = ListRequest::get_suspected(&self.list_sender).await?;
        let local_confirmed_list = ListRequest::get_confirmed(&self.list_sender).await?;

        let ping = Message::Ping
            .build_list(
                &node,
                None,
                &local_alive_list,
                &local_suspected_list,
                &local_confirmed_list,
            )
            .await;

        let mut address = member.membership_address().await;
        address.set_ip(IpAddr::from_str("127.0.0.1")?);

        info!("preparing ping target -> {:?}", &address);

        self.dissemination
            .send(Dissemination::Message(ping, address))?;

        match timeout(self.time_out, ping_target_ack.recv()).await {
            Ok(Ok(PingTarget::Member(ping_target))) => {
                info!("address -> {:?}", &address);
                info!("ping target -> {:?}", &ping_target);

                match address == ping_target {
                    true => {
                        self.alive(&member).await?;

                        Ok(())
                    }
                    false => Err(Box::from("received ack from unexpected member")),
                }
            }
            Ok(Err(error)) => Err(Box::from(error)),
            Err(error) => {
                self.suspected(&member).await?;

                Err(Box::from(error))
            }
        }
    }

    async fn send_indirect(
        &self,
        member: Node,
        suspect: Node,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut ping_target_ack = self.ping_target_channel.subscribe();

        let node = ListRequest::get_node(&self.list_sender).await?;
        let local_alive_list = ListRequest::get_alive(&self.list_sender).await?;
        let local_suspected_list = ListRequest::get_suspected(&self.list_sender).await?;
        let local_confirmed_list = ListRequest::get_confirmed(&self.list_sender).await?;

        let ping_req = Message::PingReq
            .build_list(
                &node,
                Some((
                    &node.membership_address().await,
                    &suspect.membership_address().await,
                )),
                &local_alive_list,
                &local_suspected_list,
                &local_confirmed_list,
            )
            .await;

        let mut address = member.membership_address().await;
        address.set_ip(IpAddr::from_str("127.0.0.1")?);

        warn!("preparing ping req target -> {:?}", &address);

        self.dissemination
            .send(Dissemination::Message(ping_req, address))?;

        match timeout(self.time_out, ping_target_ack.recv()).await {
            Ok(Ok(PingTarget::Member(ping_target))) => {
                match suspect.membership_address().await == ping_target {
                    true => {
                        self.alive(&suspect).await?;

                        Ok(())
                    }
                    false => Err(Box::from("received ack from unexpected member...")),
                }
            }
            Ok(Err(error)) => Err(Box::from(error)),
            Err(error) => {
                self.confirmed(&suspect).await?;

                Err(Box::from(error))
            }
        }
    }

    async fn alive(&self, member: &Node) -> Result<(), Box<dyn std::error::Error>> {
        ListRequest::remove_confirmed(&self.list_sender, member).await?;
        ListRequest::remove_suspected(&self.list_sender, member).await?;
        ListRequest::insert_alive(&self.list_sender, member).await?;

        Ok(())
    }

    async fn suspected(&self, member: &Node) -> Result<(), Box<dyn std::error::Error>> {
        ListRequest::remove_confirmed(&self.list_sender, member).await?;
        ListRequest::remove_alive(&self.list_sender, member).await?;
        ListRequest::insert_suspected(&self.list_sender, member).await?;

        Ok(())
    }

    async fn confirmed(&self, member: &Node) -> Result<(), Box<dyn std::error::Error>> {
        ListRequest::remove_alive(&self.list_sender, member).await?;
        ListRequest::remove_suspected(&self.list_sender, member).await?;
        ListRequest::insert_confirmed(&self.list_sender, member).await?;

        Ok(())
    }
}
