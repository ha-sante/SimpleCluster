use std::net::IpAddr;
use winping::{Buffer, Pinger};

use std::thread;
use tonic::{transport::Server, Request, Response, Status};

use dondon::instance_client::InstanceClient;
use dondon::instance_server::{Instance, InstanceServer};

use dondon::{HelloReply, HelloRequest};

pub mod dondon {
    tonic::include_proto!("dondon");
}

#[derive(Default)]
pub struct DondonInstance {}

#[tonic::async_trait]
impl Instance for DondonInstance {
    async fn hello(&self, request: Request<HelloRequest>) -> Result<Response<HelloReply>, Status> {
        println!("- Got a request from {:?}", request.remote_addr());

        let reply = dondon::HelloReply {
            message: format!("Hello! {}", request.into_inner().name),
        };

        Ok(Response::new(reply))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let find = find_port();
    let port = find.to_string();
    let addr = format!("[::1]:{}", port).parse().unwrap();
    let greeter = DondonInstance::default();

    let instance_thread = thread::spawn(move || async move {
        println!("1. Service Instance listening on {}", addr);
        Server::builder()
            .add_service(InstanceServer::new(greeter))
            .serve(addr)
            .await;
    });

    // find_friends(find).await;
    instance_thread.join().unwrap().await;

    println!("Logging after the call went through");

    Ok(())
}

// Scans a port in specific range and returns first open match only
fn find_port() -> u16 {
    println!("2. Finding a port for this instance");
    let mut port = 0;
    // let mut ping = Ping::new();

    // for index in 2..40 {
    //     try!(ping.add_host(format!("::1:{}", index).as_str()));  // IPv4 / IPv6 addresses OK
    // }

    // let responses = try!(ping.send());
    // for resp in responses {
    //     if resp.dropped > 0 {
    //         println!("No response from host: {}", resp.hostname);
    //     } else {
    //         println!("Response from host {} (address {}): latency {} ms",
    //             resp.hostname, resp.address, resp.latency_ms);
    //         println!("    all details: {:?}", resp);
    //     }
    // }

    // let (pinger, results) = match Pinger::new(None, Some(56)) {
    //     Ok((pinger, results)) => (pinger, results),
    //     Err(e) => panic!("Error creating pinger: {}", e),
    // };

    // pinger.add_ipaddr("8.8.8.8");
    // pinger.add_ipaddr("1.1.1.1");
    // pinger.add_ipaddr("7.7.7.7");
    // pinger.add_ipaddr("2001:4860:4860::8888");
    // pinger.run_pinger();
    // loop {
    //     match results.recv() {
    //         Ok(result) => match result {
    //             Idle { addr } => {
    //                 println!("Idle Address {}.", addr);
    //             }
    //             Receive { addr, rtt } => {
    //                 println!("Receive from Address {} in {:?}.", addr, rtt);
    //             }
    //         },
    //         Err(_) => panic!("Worker threads disconnected before the solution was found!"),
    //     }
    // }

    let pinger = Pinger::new().unwrap();
    let mut buffer = Buffer::new();

    for index in 0..4 {
        let dst = std::env::args()
            .nth(1)
            .unwrap_or(String::from(format!("::1:{}", index)))
            .parse::<IpAddr>()
            .expect("Could not parse IP Address");

        println!("Sending Ping to {}.", dst);

        match pinger.send(dst, &mut buffer) {
            Ok(rtt) => println!("Response time {} ms.", rtt),
            Err(err) => println!("{}.", err),
        }
    }

    port
}

// Scans a range of ports and returns those who respond to hello RPC calls
async fn find_friends(skip: u16) -> Result<(), Box<dyn std::error::Error>> {
    println!("3. Finding for other dondons");
    let mut others: Vec<u16> = Vec::new();
    for index in 2..7 {
        println!("- Currently at index {index}");
        // println!("- Local port {index} free ? = {}", is_free(index));

        // Call a hello rpc call, if true, then it's him/her
        // let address = format!("http://[::1]:{}", index);
        // let mut client = InstanceClient::connect(address).await?;
        // let request = tonic::Request::new(HelloRequest {
        //     name: "Tonic".into(),
        // });

        // let response = client.hello(request).await;

        // println!("RESPONSE={:?}", response);

        // if !is_local_port_free(index) {

        // } else {
        //     // println!("- Local port {index} is not free");
        // }
    }
    println!("- Found list of others");
    Ok(())
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
