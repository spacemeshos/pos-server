#[macro_use]
extern crate log;
extern crate pos_api;

use log::LevelFilter;
use pos_api::api::job::JobStatus;
use pos_api::api::{
    AddJobRequest, GetAllJobsStatusRequest, GetConfigRequest, JobStatusStreamRequest,
    SetConfigRequest,
};
use std::convert::TryInto;
use tokio_stream::StreamExt;

mod test_helpers;

/// 16 long jobs using gpu providers only and pow computation
#[tokio::test]
async fn rig_test() {
    // Minimum is 268435456. Requested 134217728

    let pow_difficulty: Vec<u8> = vec![
        0x00, 0x00, 0x00, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
        0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
        0xff, 0xff,
    ];

    let _ = env_logger::builder()
        .is_test(false)
        .filter_level(LevelFilter::Info)
        .try_init();

    let (mut api_client, guard) = test_helpers::start_server(false).await;

    // Update default config
    let mut config = api_client
        .get_config(GetConfigRequest {})
        .await
        .unwrap()
        .into_inner()
        .config
        .unwrap();

    const POST_SIZE_BITS: u64 = 32 * 1024 * 1024 * 3;

    config.indexes_per_compute_cycle = 4 * 1024 * 1024;

    let _ = api_client
        .set_config(SetConfigRequest {
            config: Some(config.clone()),
        })
        .await
        .unwrap();

    let mut receiver = api_client
        .subscribe_job_status_stream(JobStatusStreamRequest { id: 0 })
        .await
        .unwrap()
        .into_inner();

    let client_id = hex::decode("1215eda121").unwrap();
    // start 16 jobs
    let jobs_count = 16;
    for i in 0..jobs_count {
        let _ = api_client
            .add_job(AddJobRequest {
                client_id: client_id.clone(),
                post_size_bits: POST_SIZE_BITS,
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
        jobs_count,
        "expected {} queued jobs",
        jobs_count
    );

    // print status and wait for all jobs to complete
    let mut completed_jobs = 0;
    while let Some(res) = receiver.next().await {
        match res {
            Ok(job_status) => {
                let job = job_status.job.unwrap();
                match job.status.try_into().unwrap() {
                    JobStatus::Completed => {
                        info!("🎉🎉 completed. job {}", job);
                        completed_jobs += 1;
                        if completed_jobs == jobs_count {
                            info!("all jobs completed");
                            break;
                        }
                    }
                    JobStatus::Stopped => {
                        panic!("💥💥 job stopped due to error: {}", job)
                    }
                    JobStatus::Queued => {
                        info!("job queued: {}", job);
                    }
                    JobStatus::Started => {
                        info!("job in progress... {}", job);
                    }
                }
            }
            Err(e) => panic!("💥💥 server error {}", e),
        }
    }

    // delete all pos files
    test_helpers::delete_pos_files(&all_jobs_response.jobs, config.data_dir);

    // prevent the rust compiler from dropping guard before end of test
    info!("{}", guard.0.id());
}
