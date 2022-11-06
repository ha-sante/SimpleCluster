use futures::{
    future::{self, Ready},
    // prelude::*,
};
// use io::stdin;
// use std::io;
use tarpc::{
    client, context,
    server::{self, incoming::Incoming, Channel},
};

// Service Traits available to service clients
#[tarpc::service]
trait Service {
    async fn alive() -> String; // Returns a true or times out
}

// Methods Implementation called upon by service clients
#[derive(Clone)]
struct NodeService;
impl Service for NodeService {
    type AliveFut = Ready<String>;
    fn alive(self, _: context::Context) -> Self::AliveFut {
        future::ready(format!("Service is Alive"))
    }
}

// Start live listerner for client method calls
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initiate the service for recieving calls
    let (client_transport, server_transport) = tarpc::transport::channel::unbounded();
    let server = server::BaseChannel::with_defaults(server_transport);
    tokio::spawn(server.execute(NodeService.serve()));

    // An example client calling/testing a spawned service
    let client = ServiceClient::new(client::Config::default(), client_transport).spawn();
    let hello = client.alive(context::current()).await?;

    println!("Alive Check = {hello}");
    Ok(())
}

/*
Using Tarp
- Define trait for the methods available to RPC calls
- Implement the trait methods & Associate it to the Server Struct
- Implement the server itself in an init method

Learnings
- Tokio is how Tarp is capable of recieving and processing multiple RPC calls made in close succession

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
