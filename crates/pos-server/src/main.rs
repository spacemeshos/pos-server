extern crate pos_api;
extern crate pos_compute;

use pos_api::*;
use pos_compute::*;

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    do_providers_list();
    do_benchmark();
    Ok(())
}
