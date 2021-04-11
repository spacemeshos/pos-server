extern crate gpu_post_bindings as post;

use std::str;

const LABEL_SIZE: u32 = 1;
const LABELS_COUNT: u64 = 9 * 128 * 1024;

fn do_benchmark() {
    let id: [u8; 32] = [0; 32];
    let salt: [u8; 32] = [0; 32];
    let providers = post::get_providers();


    if providers.len() > 0 {
        const out_size: usize = (LABELS_COUNT as usize * LABEL_SIZE as usize + 7) / 8;
        let mut out: [u8; out_size] = [0; out_size];
        for provider in &providers {
            if provider.compute_api as u32 != post::COMPUTE_API_CLASS_CPU {
                let mut hashes_computed: u64 = 0;
                let mut hashes_per_sec: u64 = 0;
                post::scrypt_positions(
                    provider.id,
                    &id,
                    0,
                    LABELS_COUNT as u64 - 1,
                    LABEL_SIZE,
                    &salt,
                    0,
                    &mut out,
                    512,
                    1,
                    1,
                    &mut hashes_computed as *mut u64,
                    &mut hashes_per_sec as *mut u64,
                );
                println!(
                    "{}: {} hashes, {} h/s",
                    provider.model, hashes_computed, hashes_per_sec
                );
            }
        }
    }
}

fn get_provider_class_string(class: u32) -> &'static str {
    match class {
        post::COMPUTE_API_CLASS_UNSPECIFIED => "UNSPECIFIED",
        post::COMPUTE_API_CLASS_CPU => "CPU",
        post::COMPUTE_API_CLASS_CUDA => "CUDA",
        post::COMPUTE_API_CLASS_VULKAN => "VULKAN",
        _ => "INVALID",
    }
}

fn do_providers_list() {
    let providers = post::get_providers();
    println!("Available POST compute providers:");
    for provider in &providers {
        println!(
            "{}: [{}] {}",
            provider.id,
            get_provider_class_string(provider.compute_api),
            provider.model
        );
    }
}

fn main() {
    do_providers_list();
    do_benchmark();
}
