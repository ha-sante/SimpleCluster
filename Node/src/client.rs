// Copyright 2018 Google LLC
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.

use clap::Parser;
use service::{init_tracing, WorldClient};
// use std::{net::SocketAddr, time::Duration};
use tarpc::{client, context, tokio_serde::formats::Json};
use tokio::time::sleep;
use tracing::Instrument;

use std::{
    net::{IpAddr, Ipv6Addr,Ipv4Addr, SocketAddr},
    time::Duration,
};

#[derive(Parser)]
struct Flags {
    /// Sets the server address to connect to.
    // #[clap(long)]
    // server_addr: SocketAddr,
    /// Sets the name to say hello to.
    #[clap(long)]
    name: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // let flags = Flags::parse();
    init_tracing("Tarpc Example Client")?;
    println!("Running this main app");

    let server_addr = (IpAddr::V4(Ipv4Addr::LOCALHOST), 2);
    let transport = tarpc::serde_transport::tcp::connect(server_addr, Json::default);

    // WorldClient is generated by the service attribute. It has a constructor `new` that takes a
    // config and any Transport as input.
    let client = WorldClient::new(client::Config::default(), transport.await?).spawn();

    let hello = async move {
        // Send the request twice, just to be safe! ;)
        tokio::select! {
            hello1 = client.hello(context::current(), format!("{}1", "hello")) => { hello1 }
            hello2 = client.hello(context::current(), format!("{}2", "hello")) => { hello2 }
        }
    }
    .instrument(tracing::info_span!("Two Hellos"))
    .await;

    println!("Running this main 2");

    tracing::info!("Recieved this response from RPC call = {:?}", hello);

    // Let the background span processor finish.
    sleep(Duration::from_micros(1)).await;
    opentelemetry::global::shutdown_tracer_provider();

    Ok(())
}
