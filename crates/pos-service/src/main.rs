extern crate gpu_post;
use gpu_post::*;

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    do_providers_list();
    do_benchmark();
    Ok(())
}
