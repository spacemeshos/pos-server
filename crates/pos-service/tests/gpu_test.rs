#[macro_use]
extern crate log;

//extern crate nix;

extern crate pos_api;

use log::LevelFilter;
use pos_api::api::{
    AddJobRequest, GetConfigRequest, GetProvidersRequest, JobStatusStreamRequest, SetConfigRequest,
};
use std::env;

mod test_helpers;

#[tokio::test]
async fn gpu_job_test_pow() {
    const POST_SIZE_BITS: u64 = 32 * 1024 * 1024 * 3;
    const LABELS_PER_ITER: u64 = 4 * 1024 * 1024;

    let pow_difficulty: Vec<u8> = vec![
        0x00, 0x00, 0x00, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
        0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
        0xff, 0xff,
    ]; // 32 bytes pattern

    let _ = env_logger::builder()
        .is_test(false)
        .filter_level(LevelFilter::Info)
        .try_init();

    let path = env::current_dir().unwrap();
    info!("Path: {:?}", path);
    let (mut api_client, guard) = test_helpers::start_server(false).await;

    let mut config = api_client
        .get_config(GetConfigRequest {})
        .await
        .unwrap()
        .into_inner()
        .config
        .unwrap();

    config.indexes_per_compute_cycle = LABELS_PER_ITER;

    let _ = api_client
        .set_config(SetConfigRequest {
            config: Some(config.clone()),
        })
        .await
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
            post_size_bits: POST_SIZE_BITS,
            start_index: 0,
            friendly_name: "world's first pos".to_string(),
            pow_difficulty,
            compute_pow_solution: true,
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
