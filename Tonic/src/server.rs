use port_scanner::*;
use tokio::time::error::Error;
use std::net::*;
use winping::{Buffer, Pinger};


#[macro_use] extern crate airone;
use airone::prelude::*;

use std::net::Ipv6Addr;
use std::str::FromStr;
use std::thread;

use tonic::{transport::Server, Request, Response, Status};

use dondon::instance_client::InstanceClient;
use dondon::instance_server::{Instance, InstanceServer};

use dondon::{HelloReply, HelloRequest};



airone_init!();
airone_db!(
    // struct NodeInstance {
    //     pub address: String,
    //     pub friends: Vec<String>,
    //     pub group: Vec<String>,
    //     pub leader: String,
    // }
    struct Foo
    {
        pub address: f64,
        pub field2: String,
        field3: Option<i32>
    }
);

// struct NodeInstance {
//     pub address: String,
//     pub friends: Vec<String>,
//     pub group: Vec<String>,
//     pub leader: String,
// }

// struct NodeInstance {
//     port: u16,
//     friends: Vec<String>,
//     group: Vec<String>,
//     leader: u16,
// }

// Stores details of current instance
// static mut thisInstance: NodeInstance = NodeInstance {
//     port : 0,
//     friends: Vec::new(),
//     group: Vec::new(),
//     leader: 0
// };


pub mod dondon {
    tonic::include_proto!("dondon");
}

#[derive(Default)]
pub struct DondonInstance {}

#[tonic::async_trait]
impl Instance for DondonInstance {

    // Sends back hello call response
    async fn hello(&self, request: Request<HelloRequest>) -> Result<Response<HelloReply>, Status> {
        println!("- Got a request from {:?}", request.remote_addr());

        let reply = dondon::HelloReply {
            message: format!("Hello! {}", request.into_inner().name),
        };

        Ok(Response::new(reply))
    }

    // Calls for an update to the friends list of this node
    async fn find_friends(&self, request: Request<HelloRequest>) -> Result<Response<HelloReply>, Status> {
        println!("- Got a request from {:?}", request.remote_addr());

        let reply = dondon::HelloReply {
            message: format!("Hello! {}", request.into_inner().name),
        };

        // find_friends().await;

        Ok(Response::new(reply))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let port = find_port();
    let greeter = DondonInstance::default();
    let socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), port);
    let instance_thread = thread::spawn(move || async move {
        println!("1. Service Instance listening on {}", socket);
        Server::builder()
            .add_service(InstanceServer::new(greeter))
            .serve(socket)
            .await;
    });

    let mut friends:Vec<String> = Vec::new();
    let mut database: AironeDb<Foo> = AironeDb::new();

    // Find other server instances
    let list = find_friends(port, &friends).await;
    match list{
        Result => println!("5. Recieved friends list of {:?}", Result),
        _ => println!("Recieved no collection of friends")
    }

    // Start server instance
    instance_thread.join().unwrap().await;

    println!("Logging after the call went through");

    Ok(())
}

// Scans a port in specific range and returns first open match only
fn find_port() -> u16 {
    println!("2. Finding a port for this instance");
    let mut port = 2;

    for index in 0..10 {
        println!(
            "- Port {index} open? {}",
            scan_port_addr(format!("127.0.0.1:{}", index))
        );
        let port_in_use = scan_port_addr(format!("127.0.0.1:{}", index));
        if !port_in_use {
            port = index;
            break;
        }
    }

    port
}

// Scans a range of ports and returns those who respond to hello RPC calls
async fn find_friends(skip: u16, friends: &Vec<String>) -> Result<Vec<u16>, Box<dyn std::error::Error>> {
    println!("3. Finding for other dondons");
    let mut others: Vec<u16> = Vec::new();
    for index in 2..7 {
        // If port is in use
        let port_in_use = scan_port_addr(format!("127.0.0.1:{}", index));
        if port_in_use && index != skip {
            others.push(index);
        }
    }

    println!("4. Validating their dondon");
    let mut valid: Vec<u16> = Vec::new();
    for port in others {
        println!("- Validing 127.0.0.1:{} for dondoness", port);
        let mut client = InstanceClient::connect(format!("http://127.0.0.1:{}", port)).await?;

        let request = tonic::Request::new(HelloRequest {
            name: "Tonic".into(),
        });

        let response = client.hello(request).await?;

        println!("RESPONSE={:?}", response);
        valid.push(port);
        // friends.push(format!("http://127.0.0.1:{}", port))
    }

    // TODO: Store the found list into an local variable
    Ok(valid)
}


/*
Methods
- main = starts the instance & it's rpc setup
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
