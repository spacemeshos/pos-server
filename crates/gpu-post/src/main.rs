extern crate gpu_post;

use std::str;

// basic example app of using the lib
fn main() {
    gpu_post::do_providers_list();
    gpu_post::do_benchmark();
}
