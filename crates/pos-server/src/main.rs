#[macro_use]
extern crate log;
extern crate pos_api;
extern crate pos_compute;

mod api;
mod server;

use crate::server::PosServer;
use pos_compute::*;
use xactor::*;

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let _server = PosServer::from_registry().await?;

    do_providers_list();
    do_benchmark();
    Ok(())
}
