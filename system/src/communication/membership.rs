use std::net::SocketAddr;

use std::net::Ipv4Addr;

use std::net::IpAddr;

use std::str::FromStr;

use tokio::net::UdpSocket;

pub enum Messages {
    Ping,
    PingReq,
    Ack,
}

pub struct MembershipCommunication {
    socket_address: SocketAddr,
    buffer: [u8; 1024],
}

impl MembershipCommunication {
    pub async fn init(
        socket_address: SocketAddr,
    ) -> Result<MembershipCommunication, Box<dyn std::error::Error>> {
        let buffer = [0; 1024];

        Ok(MembershipCommunication {
            socket_address,
            buffer,
        })
    }

    pub async fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let socket = UdpSocket::bind(self.socket_address).await?;

        let multicast_address = Ipv4Addr::new(239, 0, 0, 1);
        let interface = Ipv4Addr::new(0, 0, 0, 0);

        socket.join_multicast_v4(multicast_address, interface)?;

        loop {
            let (bytes, origin) = socket.recv_from(&mut self.buffer).await?;

            println!("incoming bytes - {:?}", bytes);
            println!("origin - {:?}", origin);

            let len = socket.send_to(&mut self.buffer[..bytes], origin).await?;
            println!("{:?} bytes sent", len);

            println!(
                "received {:?}",
                String::from_utf8(self.buffer[..bytes].to_vec())?,
            );
        }

        Ok(())
    }
}
