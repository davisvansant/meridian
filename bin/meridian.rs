use clap::{App, Arg, SubCommand};
use std::net::IpAddr;
use std::str::FromStr;

use system::node::Node;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let leaders = Arg::with_name("cluster_size")
        .help("expected size of cluster and failure tolerance (1 leader/2 leaders)")
        .long("cluster_size")
        .takes_value(true)
        .possible_value("1")
        .possible_value("3")
        .possible_value("5")
        .required(true)
        .value_name("NUMBER")
        .require_equals(true)
        .display_order(1);
    let server_ip_address = Arg::with_name("ip_address")
        .help("set the ip address")
        .long("ip_address")
        .takes_value(true)
        .default_value("0.0.0.0")
        .value_name("ADDRESS")
        .require_equals(true)
        .display_order(2);
    let server_cluster_port = Arg::with_name("cluster_port")
        .help("set the cluster port (internal communications gRPC)")
        .long("cluster_port")
        .takes_value(true)
        .default_value("10000")
        .value_name("PORT")
        .require_equals(true)
        .display_order(3);
    let server_client_port = Arg::with_name("client_port")
        .help("set the client port (client communications gRPC)")
        .long("client_port")
        .takes_value(true)
        .default_value("20000")
        .value_name("PORT")
        .require_equals(true)
        .display_order(4);
    let server_membership_port = Arg::with_name("membership_port")
        .help("set the membership port (node join communications gRPC)")
        .long("membership_port")
        .takes_value(true)
        .default_value("25000")
        .value_name("PORT")
        .require_equals(true)
        .display_order(5);
    let run = SubCommand::with_name("run")
        .about("run meridian")
        .arg(leaders)
        .arg(server_ip_address)
        .arg(server_cluster_port)
        .arg(server_client_port)
        .arg(server_membership_port);
    let meridian = App::new("meridian")
        .author("some_author_goes_here")
        .version(env!("CARGO_PKG_VERSION"))
        .about("meridian")
        .subcommand(run)
        .get_matches();

    match meridian.subcommand() {
        ("run", Some(run)) => {
            let cluster_size = run.value_of("cluster_size").unwrap();

            let ip_address_value = run.value_of("ip_address").unwrap();
            let ip_address = IpAddr::from_str(ip_address_value)?;

            println!("set ip address - {:?}", &ip_address);

            let cluster_port_value = run.value_of("cluster_port").unwrap();
            let cluster_port = u16::from_str(cluster_port_value)?;

            println!("launching cluster on {:?}", &cluster_port);

            let client_port_value = run.value_of("client_port").unwrap();
            let client_port = u16::from_str(client_port_value)?;

            println!("launching client on {:?}", &client_port);

            let membership_port_value = run.value_of("membership_port").unwrap();
            let membership_port = u16::from_str(membership_port_value)?;

            println!("launching membership on {:?}", &membership_port);

            let node = Node::init(ip_address, cluster_port, client_port, membership_port).await?;

            node.run(cluster_size).await?;
        }
        _ => println!("{:?}", meridian.usage()),
    }

    Ok(())
}
