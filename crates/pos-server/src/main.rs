#[macro_use]
extern crate log;
extern crate pos_api;
extern crate pos_compute;

mod api;
mod server;

use crate::server::{Init, PosServer, SetConfig};
use pos_api::api::Config;
use pos_compute::*;
use xactor::*;

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let server = PosServer::from_registry().await?;
    server.call(Init {}).await??;
    server
        .call(SetConfig(Config {
            // default config
            data_dir: "./".to_string(),
            indexes_per_compute_cycle: 9 * 128 * 1024,
            bits_per_index: 8,
            salt: vec![],
        }))
        .await??;

    do_providers_list();
    do_benchmark();
    Ok(())
}
