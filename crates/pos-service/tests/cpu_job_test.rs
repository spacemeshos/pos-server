#[macro_use]
extern crate log;
extern crate pos_api;

use log::LevelFilter;
use pos_api::api::{AddJobRequest, GetConfigRequest, GetProvidersRequest, JobStatusStreamRequest};
use std::env;

mod test_helpers;

/// One simple job using cpu provider w/o pow computation
#[tokio::test]
async fn cpu_job_test_no_pow() {
    const POST_SIZE_BITS: u64 = 8192 * 64;

    let pow_difficulty: Vec<u8> = vec![
        0x00, 0x00, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
        0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
        0xff, 0xff,
    ]; // 32 bytes pattern

    let _ = env_logger::builder()
        .is_test(false)
        .filter_level(LevelFilter::Info)
        .try_init();

    let path = env::current_dir().unwrap();
    info!("Path: {:?}", path);
    let (mut api_client, guard) = test_helpers::start_server(true).await;

    let config = api_client
        .get_config(GetConfigRequest {})
        .await
        .unwrap()
        .into_inner()
        .config
        .unwrap();

    let providers = api_client
        .get_providers(GetProvidersRequest {})
        .await
        .unwrap()
        .into_inner()
        .providers;

    for p in providers {
        info!("Provider: {}", p)
    }

    // subscribe to all jobs statuses
    let receiver = api_client
        .subscribe_job_status_stream(JobStatusStreamRequest { id: 0 })
        .await
        .unwrap()
        .into_inner();

    let client_id = hex::decode("1215eda121").unwrap();

    let resp = api_client
        .add_job(AddJobRequest {
            client_id,
            post_size_bits: POST_SIZE_BITS, // 32 * 1024 * 8, //9 * 128 * 1024 * 4 * 8,
            start_index: 0,
            friendly_name: "world's first pos".to_string(),
            pow_difficulty,
            compute_pow_solution: false,
        })
        .await
        .unwrap()
        .into_inner();

    let job = resp.job.unwrap();
    info!("job info: {}", job);
    test_helpers::job_status_handler(receiver).await;

    // delete the job's pos file
    test_helpers::delete_pos_files(&vec![job], config.data_dir);

    // prevent the compiler from dropping guard before end of test
    info!("{}", guard.0.id());
}
