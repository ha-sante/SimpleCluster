use port_check::*;
use port_scanner::*;
use std::thread;
use tonic::{transport::Server, Request, Response, Status};

use hello_world::greeter_server::{Greeter, GreeterServer};
use hello_world::{HelloReply, HelloRequest};

pub mod hello_world {
    tonic::include_proto!("helloworld");
}

#[derive(Default)]
pub struct MyGreeter {}

#[tonic::async_trait]
impl Greeter for MyGreeter {
    async fn say_hello(
        &self,
        request: Request<HelloRequest>,
    ) -> Result<Response<HelloReply>, Status> {
        println!("Got a request from {:?}", request.remote_addr());

        let reply = hello_world::HelloReply {
            message: format!("Hello {}!", request.into_inner().name),
        };
        Ok(Response::new(reply))
    }
}

fn find_port() -> u16 {
    let mut port = 0;
    for index in 2..40 {
        if scan_port(index) == false {
            port = index;
            break;
        }
    }
    port
}




#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let port = find_port().to_string();
    let addr = format!("[::1]:{}", port).parse().unwrap();
    let greeter = MyGreeter::default();

    let instance_thread = thread::spawn(move || async move {
        println!("Service Instance listening on {}", addr);

        Server::builder()
            .add_service(GreeterServer::new(greeter))
            .serve(addr)
            .await;
    });

    println!("some other code");
    instance_thread.join().unwrap().await;

    println!("Logging after the call went through");

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