use clap::Parser;
use futures::{
    future::{self, Ready},
    prelude::*,
};
// use io::stdin;
// use std::io;
use tarpc::{
    client, context,
    server::{self, incoming::Incoming, Channel},
    tokio_serde::formats::Json,
};

use std::{
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr},
    os::windows::prelude::AsRawHandle,
    time::Duration,
};

use async_std::task;

use port_check::*;
use port_scanner::*;
use std::thread;

#[derive(Parser)]
struct Flags {
    /// Sets the port number to listen on.
    #[clap(long)]
    port: u16,
}

// Server definition
#[derive(Clone)]
struct NodeService(SocketAddr);

// Service Traits definition of methods available to service clients
#[tarpc::service]
trait Service {
    async fn alive() -> String; // Returns a true or times out
}

// Methods Implementation called upon by service clients
#[tarpc::server]
impl Service for NodeService {
    type AliveFut = Ready<String>;
    fn alive(self, _: context::Context) -> Self::AliveFut {
        future::ready(format!("Service is Alive"))
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("3. Client test call is being executed 1");
    let server_addr = (IpAddr::V4(Ipv4Addr::LOCALHOST), 2);
    let transport = tarpc::serde_transport::tcp::connect(server_addr, Json::default);
    // println!("- Transport used for alive call {:?}", transport.config());
    let client = ServiceClient::new(client::Config::default(), transport.await?).spawn();
    // println!("4. Client test call is has executed 2 {:?}", client);
    let hello = client.alive(context::current()).await;

    println!("- Response from alive call {:?}", hello);

    Ok(())
}
