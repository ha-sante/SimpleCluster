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
// use std::thread;

// use io_context::*;
// use self::{init_tracing, World};




use std::env;
use tracing_subscriber::{fmt::format::FmtSpan, prelude::*};

/// This is the service definition. It looks a lot like a trait definition.
/// It defines one RPC, hello, which takes one arg, name, and returns a String.
#[tarpc::service]
pub trait World {
    /// Returns a greeting for name.
    async fn hello(name: String) -> String;
}

/// Initializes an OpenTelemetry tracing subscriber with a Jaeger backend.
pub fn init_tracing(service_name: &str) -> anyhow::Result<()> {
    env::set_var("OTEL_BSP_MAX_EXPORT_BATCH_SIZE", "12");

    let tracer = opentelemetry_jaeger::new_pipeline()
        .with_service_name(service_name)
        .with_max_packet_size(2usize.pow(13))
        .install_batch(opentelemetry::runtime::Tokio)?;

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer().with_span_events(FmtSpan::NEW | FmtSpan::CLOSE))
        .with(tracing_opentelemetry::layer().with_tracer(tracer))
        .try_init()?;

    Ok(())
}



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
        println!("Alive call has been called");
        future::ready(format!("Service is Alive"))
    }
}

// Scans for the next port in the range
fn find_port() -> u16 {
    let mut port = 0;
    for index in 2..40 {
        if scan_port(index) == false {
            println!("- Port number {} available for instance use", index);
            port = index;
            break;
        }
    }
    port
}

// Starts a listener service for rpc calls
async fn start_listener() -> anyhow::Result<()> {
    // Generate valid network properties for this service
    let port = find_port();
    let server_addr = (IpAddr::V4(Ipv4Addr::LOCALHOST), port);

    // Initiate service listener to outside services
    let mut listener = tarpc::serde_transport::tcp::listen(&server_addr, Json::default).await?;
    listener.config_mut().max_frame_length(usize::MAX);
    println!(
        "- Node instance has been started on 127.0.0.1:{}",
        port
    );

    // find_friends().await;
    // TODO: Run test node service listener when the request processor has loaded
    // let (client_transport, server_transport) = tarpc::transport::channel::unbounded();
    // let server = server::BaseChannel::with_defaults(server_transport);
    // let socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 2);
    // tokio::spawn(server.execute(NodeService(socket).serve()));

    // Process Incoming requests to this service
    listener
        .filter_map(|r| future::ready(r.ok()))
        .map(server::BaseChannel::with_defaults)
        .max_channels_per_key(1, |t| t.transport().peer_addr().unwrap().ip())
        .map(|channel| async {
            let SocketAddr = channel.transport().peer_addr().unwrap();
            println!("- Channel service instance running on {}", SocketAddr);
            let server = NodeService(SocketAddr);
            channel.execute(server.serve());
        })
        .buffer_unordered(1)
        .for_each(|_| async {})
        .await;

    println!("- Crossed listener request processor");

    Ok(())
}

// Scans for all nodes which respond to alive calls
async fn find_friends() -> anyhow::Result<()> {
    // Find all valid ports which are in use
    println!("2. Scanning for alive nodes");
    let mut inuse: Vec<u16> = Vec::new();
    for index in 2..40 {
        // println!("Is port::{index} free? = {}", is_local_port_free(index));
        if is_local_port_free(index) == false {
            inuse.push(index);
        }
    }
    println!("- All inuse ports found {:?}", inuse);

    // Find all validated ports
    // Send alive call to check if service is a node
    let mut validnodes: Vec<u16> = Vec::new();
    for port in inuse {
        let server_addr = (IpAddr::V4(Ipv4Addr::LOCALHOST), port);
        let transport = tarpc::serde_transport::tcp::connect(server_addr, Json::default);
        let client = ServiceClient::new(client::Config::default(), transport.await?).spawn();
        let hello = client.alive(context::current()).await;

        println!("- Response from alive call {:?}", hello);
    }

    Ok(())
}

async fn test_node_service() -> anyhow::Result<()> {
    println!("3. Client test call is being executed 1");
    let server_addr = (IpAddr::V4(Ipv4Addr::LOCALHOST), 2);
    let transport = tarpc::serde_transport::tcp::connect(server_addr, Json::default);
    // println!("- Transport used for alive call {:?}", transport.config());
    let client = ServiceClient::new(client::Config::default(), transport.await?).spawn();
    // println!("4. Client test call is has executed 2 {:?}", client);

    // let five_seconds = Duration::new(5, 0);

    let mut ctx = tarpc::context::Context::current();
    // ctx.add_timeout(Duration::from_secs(5000));
    let hello = client.alive(ctx).await?;

    println!("- Response from alive call {:?}", hello);

    Ok(())
}

// Execute the lifecycle of node services
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing("Tarpc Example Server")?;

    start_listener().await;

    // println!("1. Spawning Node listener service");
    // let sender = thread::spawn(|| async {
    // });
    // let child = task::spawn(async {
    // });
    // // some work here
    // let res = child.await;
    // println!("- Passed waiting for child instance");
    // let receiver = thread::spawn(move || {
    //     // let value = rx.recv().expect("Unable to receive from channel");
    //     // println!("{value}");
    // });
    // sender.join().expect("The sender thread has panicked");

    Ok(())
}

/*
Using Tarp
- Define trait for the methods available to RPC calls
- Implement the trait methods & Associate it to the Server Struct
- Implement the server itself in an init method

Learnings
- Tokio is how Tarp is capable of recieving and processing multiple RPC calls made in close succession
- We need one thread responding and processing requests
- We need another sending for sending requests to other services


- returns a future instead of blocking the thread
- we need to settle the thread

Methods
- init = starts the instance & it's rpc setup
- alive = responds to rpc calls with ok
- find_friends = scans a static range of ports for alive calls
- request_leader_vote = randomly selects a node as a n5th group leader
- generate_computation = creates the jobs to be done
- request_computation = sends computation work to all group members, listens for responses from all of them and does something with the results.
- compute = takes a compute work, crunches a result and sends back as it's result
- present_computation = compiles all computed work into it's final result
- shutdown = exists a node instance and tells all friends_list to (find_friends)

Components
- friends_list = local in memory array of n5th group node memebers
- synched_global_ledger = local in memory array of all instances available across the network
*/


// function to call an api