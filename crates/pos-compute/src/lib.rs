use std::ptr;
use std::str;

pub const SPACEMESH_API_ERROR_NONE: i32 = 0;
pub const SPACEMESH_API_ERROR: i32 = -1;
pub const SPACEMESH_API_ERROR_TIMEOUT: i32 = -2;
pub const SPACEMESH_API_ERROR_ALREADY: i32 = -3;
pub const SPACEMESH_API_ERROR_CANCELED: i32 = -4;
pub const SPACEMESH_API_THROTTLED_MODE: i32 = 0x00008000;
pub const COMPUTE_API_CLASS_UNSPECIFIED: u32 = 0;
pub const COMPUTE_API_CLASS_CPU: u32 = 1; // useful for testing on systems without a cuda or vulkan GPU
pub const COMPUTE_API_CLASS_CUDA: u32 = 2;
pub const COMPUTE_API_CLASS_VULKAN: u32 = 3;

pub struct PosComputeProvider {
    pub id: u32,          // 0, 1, 2...
    pub model: String,    // e.g. Nvidia GTX 2700
    pub compute_api: u32, // A provided compute api
}

#[link(name = "gpu-setup")]
extern "C" {
    fn scryptPositions(
        provider_id: u32,          // POST compute provider ID
        id: *const u8,             // 32 bytes
        start_position: u64,       // e.g. 0
        end_position: u64,         // e.g. 49,999
        hash_len_bits: u32, // (1...256) for each hash output, the number of prefix bits (not bytes) to copy into the buffer
        salt: *const u8,    // 32 bytes
        options: u32,       // throttle etc.
        out: *mut u8, // memory buffer large enough to include hash_len_bits * number of requested hashes
        N: u32,       // scrypt N
        R: u32,       // scrypt r
        P: u32,       // scrypt p
        hashes_computed: *mut u64, //
        hashes_per_sec: *mut u64, //
    ) -> i32;

    // stop all GPU work and don't fill the passed-in buffer with any more results.
    fn stop(ms_timeout: u32, // timeout in milliseconds
    ) -> i32;

    // return non-zero if stop in progress
    fn spacemesh_api_stop_inprogress() -> i32;

    // return POST compute providers info
    fn spacemesh_api_get_providers(
        providers: *mut u8, // out providers info buffer, if NULL - return count of available providers
        max_providers: i32, // buffer size
    ) -> i32;
}

pub fn get_providers() -> Vec<PosComputeProvider> {
    unsafe {
        let p: *mut u8 = ptr::null_mut();
        let providers_count = spacemesh_api_get_providers(p, 0);
        let mut dst: Vec<PosComputeProvider> = Vec::with_capacity(providers_count as usize);
        if providers_count > 0 {
            let mut buffer: Vec<u8> = Vec::new();
            buffer.resize((providers_count * 264) as usize, 0);
            let pdst = buffer.as_mut_ptr();
            spacemesh_api_get_providers(pdst as *mut u8, providers_count);
            for i in 0..providers_count {
                let offset: usize = 264 * i as usize;
                let mut provider = PosComputeProvider {
                    id: buffer[offset + 0] as u32,
                    model: "".to_string(),
                    compute_api: buffer[offset + 260] as u32,
                };
                for j in 4..260 {
                    let c: u8 = buffer[offset + j];
                    if c == 0 {
                        break;
                    }
                    provider.model.push(c as char);
                }
                dst.push(provider);
            }
        }

        dst
    }
}

pub fn stop_inprogress() -> i32 {
    unsafe { spacemesh_api_stop_inprogress() }
}

pub fn stop_providers(ms_timeout: u32) -> i32 {
    unsafe { stop(ms_timeout) }
}

pub fn scrypt_positions(
    provider_id: u32,          // POST compute provider ID
    id: &[u8],                 // 32 bytes
    start_position: u64,       // e.g. 0
    end_position: u64,         // e.g. 49,999
    hash_len_bits: u32, // (1...256) for each hash output, the number of prefix bits (not bytes) to copy into the buffer
    salt: &[u8],        // 32 bytes
    options: u32,       // throttle etc.
    out: &mut [u8], // memory buffer large enough to include hash_len_bits * number of requested hashes
    n: u32,         // scrypt N
    r: u32,         // scrypt r
    p: u32,         // scrypt p
    hashes_computed: *mut u64, //
    hashes_per_sec: *mut u64, //
) -> i32 {
    unsafe {
        scryptPositions(
            provider_id,
            id.as_ptr(),
            start_position,
            end_position,
            hash_len_bits,
            salt.as_ptr(),
            options,
            out.as_mut_ptr(),
            n,
            r,
            p,
            hashes_computed,
            hashes_per_sec,
        )
    }
}

////////////////////////////////////////////
// Utility functions and helpers below

const LABEL_SIZE: u32 = 1;
const LABELS_COUNT: u64 = 9 * 128 * 1024;

pub fn do_benchmark() {
    let id: [u8; 32] = [0; 32];
    let salt: [u8; 32] = [0; 32];
    let providers = get_providers();

    if providers.len() > 0 {
        const out_size: usize = (LABELS_COUNT as usize * LABEL_SIZE as usize + 7) / 8;
        let mut out: [u8; out_size] = [0; out_size];
        for provider in &providers {
            if provider.compute_api as u32 != COMPUTE_API_CLASS_CPU {
                let mut hashes_computed: u64 = 0;
                let mut hashes_per_sec: u64 = 0;
                scrypt_positions(
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
        COMPUTE_API_CLASS_UNSPECIFIED => "UNSPECIFIED",
        COMPUTE_API_CLASS_CPU => "CPU",
        COMPUTE_API_CLASS_CUDA => "CUDA",
        COMPUTE_API_CLASS_VULKAN => "VULKAN",
        _ => "INVALID",
    }
}

pub fn do_providers_list() {
    let providers = get_providers();
    println!("Available pos compute providers:");
    for provider in &providers {
        println!(
            "{}: [{}] {}",
            provider.id,
            get_provider_class_string(provider.compute_api),
            provider.model
        );
    }
}