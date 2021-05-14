#[macro_use]
extern crate log;
//extern crate nix;

extern crate pos_api;

use log::LevelFilter;
use pos_api::api::job::JobStatus;
use pos_api::api::pos_data_service_client::PosDataServiceClient;
use pos_api::api::{
    AddJobRequest, GetAllJobsStatusRequest, GetConfigRequest, GetProvidersRequest, Job,
    JobStatusStreamRequest, JobStatusStreamResponse, SetConfigRequest,
};
use std::convert::TryInto;
use std::path::Path;
use std::process::{Child, Command};
use std::time::Duration;
use std::{env, fs};
use tokio::time::sleep;
use tokio_stream::StreamExt;
use tonic::transport::Channel;
use tonic::Streaming;

struct Guard(Child);

/// Child process guard
impl Drop for Guard {
    fn drop(&mut self) {
        let pid = self.0.id() as i32;
        //        match signal::kill(Pid::from_raw(pid), Signal::SIGINT) {
        match self.0.kill() {
            Err(e) => info!("could not kill child process {}: {}", pid, e),
            Ok(_) => info!("killed guarded process {}", pid),
        }
    }
}

/// Start pos server and return grpc client to its
async fn start_server(use_cpu_provider: bool) -> (PosDataServiceClient<Channel>, Guard) {
    let tests_path = env::current_dir().unwrap().join("tests");

    let config_path = match use_cpu_provider {
        true => {
            info!("using cpu provider via config file");
            tests_path.join("cpu_provider_conf.json")
        }
        false => {
            info!("using gpus providers via config file");
            tests_path.join("gpus_providers_conf.json")
        }
    };

    info!("Server config file path: {:?}", config_path);

    let server_path = "../../target/debug/pos-service";
    let server = Command::new(server_path)
        .args(&["-c", config_path.display().to_string().as_str()])
        .spawn()
        .unwrap();

    // grpc server async warmup
    sleep(Duration::from_secs(5)).await;

    (
        PosDataServiceClient::connect(format!("http://[::1]:{}", 6667))
            .await
            .expect("failed to connect to grpc server"),
        Guard(server),
    )
}

/// Multiple short jobs using cpu provider
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

    let (mut api_client, guard) = start_server(true).await;

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

    delete_pos_files(&all_jobs_response.jobs, config.data_dir);

    // prevent the compiler from dropping guard before end of test
    info!("{}", guard.0.id());
}

fn delete_pos_files(jobs: &Vec<Job>, data_dir: String) {
    for job in jobs {
        let file_name = job.file_name();
        let path = Path::new(data_dir.clone().as_str()).join(file_name);
        info!("deleting post file {}...", path.display());
        let _ = fs::remove_file(path).unwrap();
    }
}

#[tokio::test]
async fn one_job_test() {
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
    let (mut api_client, guard) = start_server(true).await;

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
        })
        .await
        .unwrap()
        .into_inner();

    let job = resp.job.unwrap();
    info!("job info: {}", job);
    job_status_handler(receiver).await;

    // delete the job's pos file
    delete_pos_files(&vec![job], config.data_dir);

    // prevent the compiler from dropping guard before end of test
    info!("{}", guard.0.id());
}

async fn job_status_handler(mut receiver: Streaming<JobStatusStreamResponse>) {
    while let Some(res) = receiver.next().await {
        match res {
            Ok(job_status) => {
                let job = job_status.job.unwrap();
                match job.status.try_into().unwrap() {
                    JobStatus::Completed => {
                        info!("🎉 completed. job {}", job);
                        break;
                    }
                    JobStatus::Stopped => {
                        panic!("💥 job {} stopped due to error", job)
                    }
                    JobStatus::Queued => {
                        info!("job {} queued", job);
                    }
                    JobStatus::Started => {
                        info!("job {} in progress", job);
                    }
                }
            }
            Err(e) => panic!("💥 server error {}", e),
        }
    }
}

/// 16 long jobs using gpu providers only
#[tokio::test]
async fn rig_test() {
    // Minimum is 268435456. Requested 134217728

    let pow_difficulty: Vec<u8> = vec![
        0x00, 0x00, 0x00, 0x00, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
        0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
        0xff, 0xff,
    ];

    let _ = env_logger::builder()
        .is_test(false)
        .filter_level(LevelFilter::Info)
        .try_init();

    let (mut api_client, guard) = start_server(true).await;

    // Update default config
    let mut config = api_client
        .get_config(GetConfigRequest {})
        .await
        .unwrap()
        .into_inner()
        .config
        .unwrap();

    const POST_SIZE_BITS: u64 = 32 * 1024 * 1024 * 8;

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
    let total_jobs = 16;
    for i in 0..total_jobs {
        let _ = api_client
            .add_job(AddJobRequest {
                client_id: client_id.clone(),
                post_size_bits: POST_SIZE_BITS,
                start_index: 0,
                friendly_name: format!("job {}", i),
                pow_difficulty: pow_difficulty.clone(),
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
    delete_pos_files(&all_jobs_response.jobs, config.data_dir);

    // prevent the rust compiler from dropping guard before end of test
    info!("{}", guard.0.id());
}
