use std::env;
use std::net::*;
use std::path;
use std::thread;

use tonic::{transport::Server, Request, Response, Status};

use dondon::instance_client::InstanceClient;
use dondon::instance_server::{Instance, InstanceServer};
use dondon::{HelloReply, HelloRequest, ComputeReply, ComputeRequest};

use port_scanner::*;
use tokio::time::error::Error;
use webbrowser;
use winping::{Buffer, Pinger};

// use evcxr::Error;
use evcxr::EvalContext;

// struct NodeInstance {
//     pub address: String,
//     pub friends: Vec<String>,
//     pub group: Vec<String>,
//     pub leader: String,
// }

#[derive(Debug)]
pub struct Friend {
    address: String,
    port: u16,
}

#[derive(Debug)]
pub struct FriendsList(Vec<Friend>);

impl FriendsList {
    pub fn create_Friend(&mut self, port_number: u16) -> &Friend {
        let new_Friend = Friend {
            address: String::from(format!("http://127.0.01:{}", port_number)),
            port: port_number,
        };

        self.0.push(new_Friend);
        return &self.0[self.0.len() - 1];
    }
}

pub mod dondon {
    tonic::include_proto!("dondon");
}

#[derive(Default)]
pub struct DondonInstance {}

#[tonic::async_trait]
impl Instance for DondonInstance {
    // Hello World RPC
    async fn hello(&self, request: Request<HelloRequest>) -> Result<Response<HelloReply>, Status> {
        println!("- Got a request from {:?}", request.remote_addr());

        let reply = dondon::HelloReply {
            message: format!("Hello! {}", request.into_inner().name),
        };

        evcxr::runtime_hook();
        if let Ok((mut context, outputs)) = EvalContext::new() {
            context.eval("let mut s = String::new();");
            context.eval(r#"s.push_str("Hello, ");"#);
            context.eval(r#"s.push_str("World!");"#);
            context.eval(r#"println!("{}", s);"#);

            // For this trivial example, we just receive a single line of output from
            // the code that was run. In a more complex case, we'd likely want to have
            // separate threads waiting for output on both stdout and stderr.
            if let Ok(line) = outputs.stdout.recv() {
                println!("{line}");
            } else {
                println!("Error running computational work")
            }
        }

        Ok(Response::new(reply))
    }

    // Friends List RPC
    async fn find_friends(
        &self,
        request: Request<HelloRequest>,
    ) -> Result<Response<HelloReply>, Status> {
        println!("- Got a request from {:?}", request.remote_addr());

        let reply = dondon::HelloReply {
            message: format!("Hello! {}", request.into_inner().name),
        };

        Ok(Response::new(reply))
    }

    // Compute RPC
    async fn compute(
        &self,
        request: Request<ComputeRequest>,
    ) -> Result<Response<ComputeReply>, Status> {
        println!("- Got a request from {:?}", request.remote_addr());

        let reply = dondon::ComputeReply {
            message: format!("Hello! {}", request.into_inner().code),
        };

        Ok(Response::new(reply))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup Executor
    evcxr::runtime_hook();

    // Server Setup
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

    // Find Friends
    let mut values: Vec<Friend> = Vec::new();
    let mut friends = FriendsList(values);
    let list = find_friends(port, &mut friends).await;
    match list {
        Result => println!("5. Recieved friends list of {:?}", friends),
        _ => println!("Recieved no collection of friends"),
    }

    // Start Compute Portal
    // TODO: Reduce redundancy if another node has opened this portal already
    let a = env::current_dir()?;
    let b = "/assets/portal.html";
    let path_result = format!("{}\n{}", a.display(), b);
    println!("Opening this page {}", path_result);
    if webbrowser::open(path_result.as_str()).is_ok() {
        // ...
    } else {
        println!("Error could not open the web portal page")
    }

    // Start Server
    instance_thread.join().unwrap().await;
    Ok(())
}

// Scans a port in specific range and returns first open match only
fn find_port() -> u16 {
    println!("2. Finding a port for this instance");
    let mut port = 2;

    for index in 2..10 {
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
async fn find_friends(
    skip: u16,
    friends: &mut FriendsList,
) -> Result<Vec<u16>, Box<dyn std::error::Error>> {
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

        let friend: Friend = Friend {
            address: format!("http://127.0.0.1:{}", port),
            port: port,
        };
        friends.0.push(friend)
    }
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
