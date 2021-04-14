#[macro_use]
extern crate log;
extern crate pos_api;
extern crate pos_compute;

mod api;
mod pos_task;
mod server;

use crate::server::{Init, PosServer, SetConfig, StartGrpcService};
use clap::{App, Arg};
use pos_api::api::Config;
use pos_compute::*;
use tokio::signal;
use xactor::*;

const DEFAULT_GRPC_PORT: u32 = 6666;
const DEFAULT_HOST: &str = "[::1]";

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let args = App::new("Pos Server")
        .version("0.1.0")
        .author("Aviv Eyal <a@spacemesh.io>")
        .about("Creates proofs of space using compute providers such as gpus")
        .arg(
            Arg::with_name("port")
                .short("p")
                .long("port")
                .takes_value(true)
                .help("set server grpc port")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("host")
                .short("h")
                .long("host")
                .takes_value(true)
                .help("set server grpc host name. e.g. [::1]")
                .takes_value(true),
        )
        .get_matches();

    let server = PosServer::from_registry().await?;
    server.call(Init {}).await??;

    // default server config - todo: take these from config file
    let salt: [u8; 32] = [0; 32];
    server
        .call(SetConfig(Config {
            // default config
            data_dir: "./".to_string(),
            indexes_per_compute_cycle: 9 * 128 * 1024,
            bits_per_index: 8,
            salt: salt.to_vec(),
            n: 512,
            r: 1,
            p: 1,
        }))
        .await??;

    let mut port = DEFAULT_GRPC_PORT;
    if let Some(v) = args.value_of("port") {
        port = v.parse().unwrap();
    }

    let mut host = DEFAULT_HOST;
    if let Some(v) = args.value_of("host") {
        host = v;
    }

    info!("Server starting...");

    server
        .call(StartGrpcService {
            port,
            host: host.into(),
        })
        .await??;

    signal::ctrl_c()
        .await
        .expect("failed to listen for ctrl-c signal");

    do_providers_list();
    do_benchmark();
    Ok(())
}
