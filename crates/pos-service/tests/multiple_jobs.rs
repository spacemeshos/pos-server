#[macro_use]
extern crate log;
extern crate pos_api;

use log::LevelFilter;
use pos_api::api::job::JobStatus;
use pos_api::api::{
    AddJobRequest, GetAllJobsStatusRequest, GetConfigRequest, JobStatusStreamRequest,
};
use std::convert::TryInto;
use tokio_stream::StreamExt;

mod test_helpers;

/// Multiple short jobs using cpu provider with pow computation
#[tokio::test]
async fn multiple_jobs_test() {
    const POST_SIZE_BITS: u64 = 8192 * 32;

    let pow_difficulty: Vec<u8> = vec![
        0x00, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
        0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
        0xff, 0xff,
    ]; // 32 bytes pattern

    let _ = env_logger::builder()
        .is_test(false)
        .filter_level(LevelFilter::Info)
        .try_init();

    let (mut api_client, guard) = test_helpers::start_server(true).await;

    let mut receiver = api_client
        .subscribe_job_status_stream(JobStatusStreamRequest { id: 0 })
        .await
        .unwrap()
        .into_inner();

    let client_id = hex::decode("1215eda121").unwrap();
    // start n jobs
    let total_jobs = 3;
    for i in 0..total_jobs {
        let _ = api_client
            .add_job(AddJobRequest {
                client_id: client_id.clone(),
                post_size_bits: POST_SIZE_BITS, // 8192, 32 * 1024 * 8, // 9 * 128 * 1024 * 4 * 8,
                start_index: 0,
                friendly_name: format!("job {}", i),
                pow_difficulty: pow_difficulty.clone(),
                compute_pow_solution: true,
            })
            .await;
    }

    let all_jobs_response = api_client
        .get_all_jobs_statuses(GetAllJobsStatusRequest {})
        .await
        .unwrap()
        .into_inner();

    assert_eq!(
        all_jobs_response.jobs.len(),
        total_jobs,
        "expected {} queued jobs",
        total_jobs
    );

    // print status and wait for all jobs to complete
    let mut completed_jobs = 0;
    while let Some(res) = receiver.next().await {
        match res {
            Ok(job_status) => {
                let job = job_status.job.unwrap();
                match job.status.try_into().unwrap() {
                    JobStatus::Completed => {
                        info!("🎉 completed. job {}", job);
                        completed_jobs += 1;
                        if completed_jobs == total_jobs {
                            info!("all jobs completed");
                            break;
                        }
                    }
                    JobStatus::Stopped => {
                        panic!("💥 job stopped due to error: {}", job)
                    }
                    JobStatus::Queued => {
                        info!("job queued: {}", job);
                    }
                    JobStatus::Started => {
                        info!("job in progress... {}", job);
                    }
                }
            }
            Err(e) => panic!("💥 server error {}", e),
        }
    }

    // delete all pos files

    let config = api_client
        .get_config(GetConfigRequest {})
        .await
        .unwrap()
        .into_inner()
        .config
        .unwrap();

    test_helpers::delete_pos_files(&all_jobs_response.jobs, config.data_dir);

    // prevent the compiler from dropping guard before end of test
    info!("{}", guard.0.id());
}
