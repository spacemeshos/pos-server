extern crate gpu_post;
extern crate pos_api;

use gpu_post::*;
use pos_api::*;

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    do_providers_list();
    do_benchmark();
    Ok(())
}
