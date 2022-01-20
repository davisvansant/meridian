use std::collections::HashMap;
use std::net::SocketAddr;

use crate::communication::membership_maintenance::GroupMember;

pub struct Dissemination {
    suspected: HashMap<SocketAddr, GroupMember>,
    confirmed: HashMap<SocketAddr, GroupMember>,
}

impl Dissemination {
    pub async fn init() -> Dissemination {
        let suspected = HashMap::with_capacity(10);
        let confirmed = HashMap::with_capacity(10);

        Dissemination {
            suspected,
            confirmed,
        }
    }

    async fn add_confirmed(&mut self, socket_address: SocketAddr, group_member: GroupMember) {
        println!(
            "adding socket address {:?} as suspect {:?}",
            &socket_address, &group_member,
        );

        if let Some(value) = self.confirmed.insert(socket_address, group_member) {
            println!("updated confirmed group member -> {:?}", value);
        } else {
            println!("suspected confirmed!");
        }
    }

    async fn add_suspected(&mut self, socket_address: SocketAddr, group_member: GroupMember) {
        println!(
            "adding socket address {:?} as suspected {:?}",
            &socket_address, &group_member,
        );

        if let Some(value) = self.suspected.insert(socket_address, group_member) {
            println!("updated suspected member group -> {:?}", value);
        } else {
            println!("suspected updated!");
        }
    }

    async fn remove_confirmed(&mut self, socket_address: &SocketAddr) {
        if let Some(remove_confirmed) = self.confirmed.remove(socket_address) {
            println!("removed from confirmed group - > {:?}", remove_confirmed);
        }
    }

    async fn remove_suspected(&mut self, socket_address: &SocketAddr) {
        if let Some(remove_suspected) = self.suspected.remove(socket_address) {
            println!("removed from suspected group - > {:?}", remove_suspected);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr};

    #[tokio::test(flavor = "multi_thread")]
    async fn init() -> Result<(), Box<dyn std::error::Error>> {
        let test_dissemination = Dissemination::init().await;

        assert!(test_dissemination.suspected.is_empty());
        assert!(test_dissemination.confirmed.is_empty());

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn add_confirmed() -> Result<(), Box<dyn std::error::Error>> {
        let test_socket_address = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 8888);
        let mut test_group_member = GroupMember::init().await;
        let mut test_dissemination = Dissemination::init().await;

        assert!(test_dissemination.confirmed.is_empty());

        test_group_member.confirm().await;

        test_dissemination
            .add_confirmed(test_socket_address, test_group_member)
            .await;

        assert_eq!(test_dissemination.confirmed.len(), 1);

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn add_suspected() -> Result<(), Box<dyn std::error::Error>> {
        let test_socket_address = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 8888);
        let mut test_group_member = GroupMember::init().await;
        let mut test_dissemination = Dissemination::init().await;

        assert!(test_dissemination.suspected.is_empty());

        test_group_member.suspect().await;

        test_dissemination
            .add_suspected(test_socket_address, test_group_member)
            .await;

        assert_eq!(test_dissemination.suspected.len(), 1);

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn remove_confirmed() -> Result<(), Box<dyn std::error::Error>> {
        let test_socket_address = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 8888);
        let mut test_group_member = GroupMember::init().await;
        let mut test_dissemination = Dissemination::init().await;

        assert!(test_dissemination.confirmed.is_empty());

        test_group_member.confirm().await;

        test_dissemination
            .add_confirmed(test_socket_address, test_group_member)
            .await;

        assert_eq!(test_dissemination.confirmed.len(), 1);

        test_dissemination
            .remove_confirmed(&test_socket_address)
            .await;

        assert_eq!(test_dissemination.confirmed.len(), 0);

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn remove_suspect() -> Result<(), Box<dyn std::error::Error>> {
        let test_socket_address = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 8888);
        let mut test_group_member = GroupMember::init().await;
        let mut test_dissemination = Dissemination::init().await;

        assert!(test_dissemination.suspected.is_empty());

        test_group_member.suspect().await;

        test_dissemination
            .add_suspected(test_socket_address, test_group_member)
            .await;

        assert_eq!(test_dissemination.suspected.len(), 1);

        test_dissemination
            .remove_suspected(&test_socket_address)
            .await;

        assert_eq!(test_dissemination.suspected.len(), 0);

        Ok(())
    }
}
