/* -------------------------------------------------------------------------- *\
 *             Apache 2.0 License Copyright © 2022 The Aurae Authors          *
 *                                                                            *
 *                +--------------------------------------------+              *
 *                |   █████╗ ██╗   ██╗██████╗  █████╗ ███████╗ |              *
 *                |  ██╔══██╗██║   ██║██╔══██╗██╔══██╗██╔════╝ |              *
 *                |  ███████║██║   ██║██████╔╝███████║█████╗   |              *
 *                |  ██╔══██║██║   ██║██╔══██╗██╔══██║██╔══╝   |              *
 *                |  ██║  ██║╚██████╔╝██║  ██║██║  ██║███████╗ |              *
 *                |  ╚═╝  ╚═╝ ╚═════╝ ╚═╝  ╚═╝╚═╝  ╚═╝╚══════╝ |              *
 *                +--------------------------------------------+              *
 *                                                                            *
 *                         Distributed Systems Runtime                        *
 *                                                                            *
 * -------------------------------------------------------------------------- *
 *                                                                            *
 *   Licensed under the Apache License, Version 2.0 (the "License");          *
 *   you may not use this file except in compliance with the License.         *
 *   You may obtain a copy of the License at                                  *
 *                                                                            *
 *       http://www.apache.org/licenses/LICENSE-2.0                           *
 *                                                                            *
 *   Unless required by applicable law or agreed to in writing, software      *
 *   distributed under the License is distributed on an "AS IS" BASIS,        *
 *   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. *
 *   See the License for the specific language governing permissions and      *
 *   limitations under the License.                                           *
 *                                                                            *
\* -------------------------------------------------------------------------- */

#![warn(clippy::unwrap_used)]

use auraed::*;
use clap::Parser;
use tonic::transport::ServerTlsConfig;
use tonic::{transport::Channel, Request};

// use crate::testclient::{observe_client::ObserveClient, StdoutRequest};
// use crate::observe_client::ObserveClient;

mod meta;
mod observe;
mod runtime;

use crate::observe::observe_client::ObserveClient;
use crate::observe::StatusRequest;

pub const OBSERVE_CLIENT_PORT: &str = "50051";
pub const OBSERVE_CLIENT_ADDR: &str = "fe80::2%vm-br0";

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct DevClientOptions {
    #[clap(short, long, value_parser, default_value = OBSERVE_CLIENT_ADDR)]
    address: String,

    #[clap(short, long, value_parser, default_value = OBSERVE_CLIENT_PORT)]
    port: String,

    #[clap(short, long)]
    verbose: bool,
}

async fn get_log_stream(
    client: &mut ObserveClient<Channel>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut stream = client
        .stdout(Request::new(observe::StdoutRequest {
            channel: "dummy".to_string(),
        }))
        .await?
        .into_inner();
    println!("Request send!");

    while let Some(logitem) = stream.message().await? {
        println!("{}: {}", logitem.timestamp, logitem.line);
    }
    println!("Stream End!");

    Ok(())
}

async fn get_status(
    client: &mut ObserveClient<Channel>,
) -> Result<(), Box<dyn std::error::Error>> {
    let response = client
        .status(Request::new(StatusRequest {
            meta: vec![meta::AuraeMeta {
                name: "UNKNOWN_NAME".to_string(),
                code: 0,
                message: "UNKNOWN_MESSAGE".to_string(),
            }],
        }))
        .await?;

    println!("Response: {:?}", response);

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let options = DevClientOptions::parse();

    let port = options.port;
    let address = options.address;

    println!("Connecting Client...");
    println!("\t address: {}", address);
    println!("\t port: {}", port);

    let mut client =
        ObserveClient::connect(format!("http://[{}]:{}", address, port))
            .await?;
    println!("Connection established!");

    println!("Get Log Stream");
    if let Err(e) = get_log_stream(&mut client).await {
        println!("Error in log stream: {:?}", e);
    }

    println!("Get Status");
    if let Err(e) = get_status(&mut client).await {
        println!("Error in status api: {:?}", e);
    }

    println!("Bye!");
    Ok(())
}
