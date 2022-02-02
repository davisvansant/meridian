use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

use tokio::net::UdpSocket;
use tokio::sync::mpsc;
use tokio::time::timeout_at;
use tokio::time::Instant;

use dissemination::Dissemination;
use suspicion::GroupMember;

use crate::channel::{
    MembershipMaintenanceReceiver, MembershipMaintenanceRequest, MembershipMaintenanceSender,
    MembershipMaintenanceShutdown,
};

mod dissemination;
mod suspicion;

#[derive(Debug, PartialEq)]
pub enum Message {
    Ack,
    Ping,
    PingReq,
}

impl Message {
    pub async fn build(&self) -> &[u8] {
        match self {
            Message::Ack => "ack".as_bytes(),
            Message::Ping => "ping".as_bytes(),
            Message::PingReq => "ping-req".as_bytes(),
        }
    }
    pub async fn from_bytes(bytes: &[u8]) -> Message {
        match bytes {
            b"ack" => Message::Ack,
            b"ping" => Message::Ping,
            b"ping-req" => Message::PingReq,
            _ => panic!("cannot build requested bytes into message"),
        }
    }
}

pub struct MembershipMaintenance {
    socket_address: SocketAddr,
    buffer: [u8; 1024],
    dissemination: Dissemination,
    receiver: MembershipMaintenanceReceiver,
    sender: MembershipMaintenanceSender,
}

impl MembershipMaintenance {
    pub async fn init(
        socket_address: SocketAddr,
        receiver: MembershipMaintenanceReceiver,
        sender: MembershipMaintenanceSender,
    ) -> Result<MembershipMaintenance, Box<dyn std::error::Error>> {
        let buffer = [0; 1024];
        let dissemination = Dissemination::init().await;

        Ok(MembershipMaintenance {
            socket_address,
            buffer,
            dissemination,
            receiver,
            sender,
        })
    }

    pub async fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let socket = UdpSocket::bind(self.socket_address).await?;

        let incoming_udp_socket = Arc::new(socket);
        let failure_detector = incoming_udp_socket.clone();

        // let (tx, mut rx) = mpsc::channel::<(Vec<u8>, SocketAddr)>(64);
        let (mut send_failure_dectector_shutdown, mut receive_failure_dectector_shutdown) =
            mpsc::channel::<MembershipMaintenanceShutdown>(1);
        let (mut send_receive_udp_message_shutdown, mut receive_receive_udp_message_shutdown) =
            mpsc::channel::<MembershipMaintenanceShutdown>(1);

        tokio::spawn(async move {
            // let mut attemps = 0;

            // while attemps <= 2 {
            while receive_failure_dectector_shutdown.recv().await.is_some() {
                if let Err(error) = MembershipMaintenance::failure_detector(&failure_detector).await
                {
                    println!("error sending udp message -> {:?}", error);
                }

                // attemps += 1;
            }
        });

        let mut buffer = self.buffer;

        tokio::spawn(async move {
            // let mut attemps = 0;

            // while attemps <= 5 {
            while receive_receive_udp_message_shutdown.recv().await.is_some() {
                if let Err(error) =
                    MembershipMaintenance::receive_udp_message(&incoming_udp_socket, &mut buffer)
                        .await
                {
                    println!("error receiving udp message -> {:?}", error);
                }

                // attemps += 1;
            }
        });

        // while let Some((bytes, origin)) = rx.recv().await {
        while let Some((request, response)) = self.receiver.recv().await {
            match request {
                MembershipMaintenanceRequest::Message((bytes, origin)) => {
                    println!("do stuff with bytes - {:?}", bytes);
                    println!("do stuff with origin -> {:?}", origin);

                    let mut group_member = GroupMember::init().await;

                    match bytes.len() {
                        5 => {
                            group_member.suspect().await;

                            self.dissemination.remove_confirmed(&origin).await;
                            self.dissemination.add_suspected(origin, group_member).await;
                        }
                        6 => {
                            group_member.confirm().await;

                            self.dissemination.remove_suspected(&origin).await;
                            self.dissemination.add_confirmed(origin, group_member).await;
                        }
                        7 => {
                            group_member.alive().await;

                            self.dissemination.remove_confirmed(&origin).await;
                            self.dissemination.remove_suspected(&origin).await;
                        }
                        _ => panic!("this will go away...setting up initial logic..."),
                    }
                }
                MembershipMaintenanceRequest::Shutdown => {
                    println!("shutting down failure dectector component...");
                    drop(send_failure_dectector_shutdown.to_owned());

                    println!("shutting down receive udp message component...");
                    drop(send_receive_udp_message_shutdown.to_owned());

                    self.receiver.close();
                    println!("membership maintenance shutdown...");
                }
            }
            // println!("do stuff with bytes - {:?}", bytes);
            // println!("do stuff with origin -> {:?}", origin);

            // let mut group_member = GroupMember::init().await;

            // match bytes.len() {
            //     5 => {
            //         group_member.suspect().await;

            //         self.dissemination.remove_confirmed(&origin).await;
            //         self.dissemination.add_suspected(origin, group_member).await;
            //     }
            //     6 => {
            //         group_member.confirm().await;

            //         self.dissemination.remove_suspected(&origin).await;
            //         self.dissemination.add_confirmed(origin, group_member).await;
            //     }
            //     7 => {
            //         group_member.alive().await;

            //         self.dissemination.remove_confirmed(&origin).await;
            //         self.dissemination.remove_suspected(&origin).await;
            //     }
            //     _ => panic!("this will go away...setting up initial logic..."),
            // }
        }

        Ok(())
    }

    async fn send_message(
        message: Message,
        socket: &UdpSocket,
        address: SocketAddr,
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!("sending -> {:?}", &message);
        println!("socket -> {:?}", &socket);
        println!("remote address -> {:?}", &address);

        let data = message.build().await;

        socket.send_to(data, address).await?;

        Ok(())
    }

    async fn receive_udp_message(
        incoming: &UdpSocket,
        buffer: &mut [u8; 1024],
    ) -> Result<(), Box<dyn std::error::Error>> {
        let (bytes, origin) = incoming.recv_from(buffer).await?;

        println!("incoming bytes - {:?}", bytes);
        println!("origin - {:?}", origin);

        let message = Message::from_bytes(&buffer[..bytes]).await;

        match message {
            Message::Ack => println!("received ack!"),
            Message::Ping => {
                println!("received ping!");

                // sender.send((Message::Ack, origin)).await?;
                let ack = Message::Ack.build().await;

                incoming.send_to(ack, origin).await?;
            }
            Message::PingReq => {
                println!("received ping req!");

                let suspected = SocketAddr::from_str("127.0.0.1:25055")?;

                // sender.send((Message::Ping, suspected)).await?;
                MembershipMaintenance::send_message(Message::Ping, incoming, suspected).await?;

                let received_ack = Message::Ack.build().await; // for now...

                incoming.send_to(received_ack, origin).await?;
            }
        }

        Ok(())
    }

    async fn failure_detector(
        failure_detector: &UdpSocket,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let random_member = SocketAddr::from_str("127.0.0.1:25200")?;
        // let ping = Message::Ping;
        let ping = Message::Ping.build().await;

        let mut buffer = [0; 1024];

        failure_detector.send_to(ping, random_member).await?;

        let receive_ack = failure_detector.recv_from(&mut buffer);

        match timeout_at(Instant::now() + Duration::from_secs(1), receive_ack).await {
            Ok(Ok((bytes, remote_origin))) => {
                println!("member is alive!");
                println!("received bytes -> {:?}", bytes);
                println!("origin -> {:?}", remote_origin);

                return Ok(());
            }
            Ok(Err(error)) => println!("error sending ping -> {:?}", error),
            Err(error) => {
                println!("error receiving ack -> {:?}", error);
                println!("sending ping-req to another member...");
            }
        }

        let another_member = SocketAddr::from_str("127.0.0.1:25202")?; //for now
        let ping_req = Message::PingReq;

        MembershipMaintenance::send_message(ping_req, failure_detector, another_member).await?;

        let receive_ping_req_ack = failure_detector.recv_from(&mut buffer);

        match timeout_at(
            Instant::now() + Duration::from_secs(1),
            receive_ping_req_ack,
        )
        .await
        {
            Ok(Ok((bytes, remote_origin))) => {
                println!("member is alive!");
                println!("received bytes -> {:?}", bytes);
                println!("origin -> {:?}", remote_origin);

                return Ok(());
            }
            Ok(Err(error)) => println!("error sending ping-req -> {:?}", error),
            Err(error) => {
                println!("error receiving ack -> {:?}", error);
                println!("no ack received from member -> {:?}", &another_member);
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::channel::{MembershipMaintenanceRequest, MembershipMaintenanceResponse};
    use std::net::{IpAddr, Ipv4Addr};
    use tokio::sync::{mpsc, oneshot};

    #[tokio::test(flavor = "multi_thread")]
    async fn message_ack() -> Result<(), Box<dyn std::error::Error>> {
        let test_message_ack = Message::Ack.build().await;

        assert_eq!(test_message_ack, b"ack");
        assert_eq!(test_message_ack.len(), 3);

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn message_ping() -> Result<(), Box<dyn std::error::Error>> {
        let test_message_ping = Message::Ping.build().await;

        assert_eq!(test_message_ping, b"ping");
        assert_eq!(test_message_ping.len(), 4);

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn message_ping_req() -> Result<(), Box<dyn std::error::Error>> {
        let test_message_ping_req = Message::PingReq.build().await;

        assert_eq!(test_message_ping_req, b"ping-req");
        assert_eq!(test_message_ping_req.len(), 8);

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn message_from_ack_bytes() -> Result<(), Box<dyn std::error::Error>> {
        let test_ack_bytes = b"ack"; //bar
        let test_message_ack = Message::from_bytes(test_ack_bytes).await;

        assert_eq!(test_message_ack, Message::Ack);

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn message_from_ping_bytes() -> Result<(), Box<dyn std::error::Error>> {
        let test_ping_bytes = b"ping";
        let test_message_ping = Message::from_bytes(test_ping_bytes).await;

        assert_eq!(test_message_ping, Message::Ping);

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn message_from_ping_req_bytes() -> Result<(), Box<dyn std::error::Error>> {
        let test_ping_req_bytes = b"ping-req";
        let test_message_ping_req = Message::from_bytes(test_ping_req_bytes).await;

        assert_eq!(test_message_ping_req, Message::PingReq);

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn init() -> Result<(), Box<dyn std::error::Error>> {
        let test_socket_address = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 8888);
        let (test_sender, test_receiver) = mpsc::channel::<(
            MembershipMaintenanceRequest,
            oneshot::Sender<MembershipMaintenanceResponse>,
        )>(64);
        let test_membership_maintenance =
            MembershipMaintenance::init(test_socket_address, test_receiver, test_sender).await?;

        assert!(test_membership_maintenance.socket_address.ip().is_ipv4());
        assert_eq!(test_membership_maintenance.buffer, [0_u8; 1024]);

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn failure_detector_ack() -> Result<(), Box<dyn std::error::Error>> {
        let test_remote_socket_address =
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 25200);

        let test_receiver = tokio::spawn(async move {
            let test_remote_socket = UdpSocket::bind(test_remote_socket_address).await.unwrap();
            let mut test_buffer = [0; 1024];

            let (test_bytes, test_origin) = test_remote_socket
                .recv_from(&mut test_buffer)
                .await
                .unwrap();

            let test_ack = Message::Ack.build().await;

            test_remote_socket
                .send_to(test_ack, test_origin)
                .await
                .unwrap();

            let test_data = Message::from_bytes(&test_buffer[..test_bytes]).await;

            assert_eq!(test_data, Message::Ping);
            assert_eq!(test_origin.to_string().as_str(), "127.0.0.1:8888");
        });

        let test_socket_address = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8888);
        // let test_membership_maintenance = MembershipMaintenance::init(test_socket_address).await?;

        // let test_socket = UdpSocket::bind(test_membership_maintenance.socket_address).await?;
        let test_socket = UdpSocket::bind(test_socket_address).await?;
        let test_failure_detector = MembershipMaintenance::failure_detector(&test_socket).await;

        assert!(test_receiver.await.is_ok());
        assert!(test_failure_detector.is_ok());

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn failure_detector_ping_req() -> Result<(), Box<dyn std::error::Error>> {
        let test_remote_socket_address =
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 25202);

        let test_receiver = tokio::spawn(async move {
            let test_remote_socket = UdpSocket::bind(test_remote_socket_address).await.unwrap();
            let mut test_buffer = [0; 1024];

            let (_test_bytes, _test_origin) = test_remote_socket
                .recv_from(&mut test_buffer)
                .await
                .unwrap();

            let test_ping = Message::Ping.build().await;

            let test_failed_remote_socket_address =
                SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 25200);

            test_remote_socket
                .send_to(test_ping, test_failed_remote_socket_address)
                .await
                .unwrap();

            let test_receive_ack = test_remote_socket.recv_from(&mut test_buffer);
            let test_timeout =
                timeout_at(Instant::now() + Duration::from_secs(2), test_receive_ack).await;

            assert!(test_timeout.is_err());
        });

        let test_socket_address = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8888);
        // let test_membership_maintenance = MembershipMaintenance::init(test_socket_address).await?;

        // let test_socket = UdpSocket::bind(test_membership_maintenance.socket_address).await?;
        let test_socket = UdpSocket::bind(test_socket_address).await?;

        let test_failure_detector = MembershipMaintenance::failure_detector(&test_socket).await;

        assert!(test_receiver.await.is_ok());
        assert!(test_failure_detector.is_ok());

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn send_message() -> Result<(), Box<dyn std::error::Error>> {
        let test_local_socket_address =
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8888);
        let test_remote_socket_address =
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8889);

        let test_receiver = tokio::spawn(async move {
            let test_remote_socket = UdpSocket::bind(test_remote_socket_address).await.unwrap();
            let mut test_buffer = [0; 1024];

            let (test_bytes, test_origin) = test_remote_socket
                .recv_from(&mut test_buffer)
                .await
                .unwrap();

            let test_data = Message::from_bytes(&test_buffer[..test_bytes]).await;

            assert_eq!(test_data, Message::Ping);
            assert_eq!(test_origin.to_string().as_str(), "127.0.0.1:8888");
        });

        let test_local_socket = UdpSocket::bind(test_local_socket_address).await?;

        MembershipMaintenance::send_message(
            Message::Ping,
            &test_local_socket,
            test_remote_socket_address,
        )
        .await?;

        assert!(test_receiver.await.is_ok());

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn receive_udp_message_ping() -> Result<(), Box<dyn std::error::Error>> {
        let test_receiver_socket_address =
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8888);
        // let test_receiver_membership_maintenance =
        //     MembershipMaintenance::init(test_receiver_socket_address).await?;

        let test_receiver = tokio::spawn(async move {
            // let test_receiver_socket = UdpSocket::bind(test_receiver_socket_address).await.unwrap();
            let mut test_receiver_buffer = [0; 1024];
            // let test_receiver_socket = UdpSocket::bind(test_receiver_socket_address).await.unwrap();
            let test_receiver_socket = UdpSocket::bind(test_receiver_socket_address).await.unwrap();
            // let mut test_receiver_buffer = test_receiver_membership_maintenance.buffer;

            let test_receive_ack = MembershipMaintenance::receive_udp_message(
                &test_receiver_socket,
                &mut test_receiver_buffer,
            )
            .await;

            assert!(test_receive_ack.is_ok());
        });

        let test_sender_socket_address =
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8889);
        // let mut test_sender_membership_maintenance =
        //     MembershipMaintenance::init(test_sender_socket_address).await?;

        // let test_sender_socket = UdpSocket::bind(test_sender_socket_address).await?;
        let test_sender_socket = UdpSocket::bind(test_sender_socket_address).await?;
        let mut test_receiver_buffer = [0; 1024];
        let test_ping = Message::Ping.build().await;

        test_sender_socket
            .send_to(test_ping, &test_receiver_socket_address)
            .await?;

        // let (test_bytes, test_origin) = test_sender_socket
        //     .recv_from(&mut test_sender_membership_maintenance.buffer)
        //     .await?;
        let (test_bytes, test_origin) = test_sender_socket
            .recv_from(&mut test_receiver_buffer)
            .await?;

        // let test_receive_ack =
        //     Message::from_bytes(&test_sender_membership_maintenance.buffer[..test_bytes]).await;
        let test_receive_ack = Message::from_bytes(&test_receiver_buffer[..test_bytes]).await;

        assert!(test_receiver.await.is_ok());
        // assert!(test_send_ping.is_ok());
        assert_eq!(test_origin.to_string().as_str(), "127.0.0.1:8888");
        assert_eq!(test_receive_ack, Message::Ack);

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn receive_udp_message_ping_req() -> Result<(), Box<dyn std::error::Error>> {
        let test_ping_req_receiver_socket_address =
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8888);
        // let test_ping_req_receiver_membership_maintenance =
        //     MembershipMaintenance::init(test_ping_req_receiver_socket_address).await?;

        let test_ping_req_receiver = tokio::spawn(async move {
            let test_receiver_socket = UdpSocket::bind(test_ping_req_receiver_socket_address)
                .await
                .unwrap();
            // let mut test_receiver_buffer = test_ping_req_receiver_membership_maintenance.buffer;
            let mut test_receiver_buffer = [0; 1024];

            let test_receive_ping_req = MembershipMaintenance::receive_udp_message(
                &test_receiver_socket,
                &mut test_receiver_buffer,
            )
            .await;

            assert!(test_receive_ping_req.is_ok());
        });

        let test_suspected_socket_address =
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 25055);
        // let test_suspected_membership_maintenance =
        //     MembershipMaintenance::init(test_suspected_socket_address).await?;

        let test_suspected_receiver = tokio::spawn(async move {
            let test_receiver_socket = UdpSocket::bind(test_suspected_socket_address)
                .await
                .unwrap();
            // let mut test_receiver_buffer = test_suspected_membership_maintenance.buffer;
            let mut test_receiver_buffer = [0; 1024];
            let test_receive_ping = test_receiver_socket
                .recv_from(&mut test_receiver_buffer)
                .await
                .unwrap();
            let test_ping = Message::from_bytes(&test_receiver_buffer[..test_receive_ping.0]).await;

            assert_eq!(test_ping, Message::Ping);
        });

        let test_sender_socket_address =
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8889);
        // let mut test_sender_membership_maintenance =
        //     MembershipMaintenance::init(test_sender_socket_address).await?;
        let mut test_sender_buffer = [0; 1024];

        let test_sender_socket = UdpSocket::bind(test_sender_socket_address).await?;
        let test_ping_req = Message::PingReq.build().await;

        test_sender_socket
            .send_to(test_ping_req, &test_ping_req_receiver_socket_address)
            .await?;

        // let (test_bytes, test_origin) = test_sender_socket
        //     .recv_from(&mut test_sender_membership_maintenance.buffer)
        //     .await?;
        let (test_bytes, test_origin) = test_sender_socket
            .recv_from(&mut test_sender_buffer)
            .await?;

        // let test_receive_ack =
        //     Message::from_bytes(&test_sender_membership_maintenance.buffer[..test_bytes]).await;
        let test_receive_ack = Message::from_bytes(&test_sender_buffer[..test_bytes]).await;

        assert!(test_ping_req_receiver.await.is_ok());
        assert!(test_suspected_receiver.await.is_ok());
        assert_eq!(test_origin.to_string().as_str(), "127.0.0.1:8888");
        assert_eq!(test_receive_ack, Message::Ack);

        Ok(())
    }
}
