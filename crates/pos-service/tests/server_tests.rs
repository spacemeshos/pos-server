#[macro_use]
extern crate log;
extern crate nix;

extern crate pos_api;

use log::LevelFilter;
use nix::sys::signal;
use nix::sys::signal::Signal;
use nix::unistd::Pid;
use pos_api::api::job::JobStatus;
use pos_api::api::pos_data_service_client::PosDataServiceClient;
use pos_api::api::{
    AddJobRequest, GetAllJobsStatusRequest, GetConfigRequest, GetProvidersRequest, Job,
    JobStatusStreamRequest, JobStatusStreamResponse,
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
        match signal::kill(Pid::from_raw(pid), Signal::SIGINT) {
            Err(e) => info!("could not kill child process {}: {}", pid, e),
            Ok(_) => info!("killed process {}", pid),
        }
    }
}

/// Start pos server and return grpc client to its
async fn start_server() -> (PosDataServiceClient<Channel>, Guard) {
    let path = env::current_dir().unwrap();
    info!("Path: {:?}", path);

    let server_path = "../../target/debug/pos-service";
    let server = Command::new(server_path).spawn().unwrap();

    // grpc server async warmup
    sleep(Duration::from_secs(5)).await;

    (
        PosDataServiceClient::connect(format!("http://[::1]:{}", 6667))
            .await
            .expect("failed to connect to grpc server"),
        Guard(server),
    )
}

#[tokio::test]
async fn multiple_jobs_test() {
    let _ = env_logger::builder()
        .is_test(false)
        .filter_level(LevelFilter::Info)
        .try_init();

    let (mut api_client, guard) = start_server().await;

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
                post_size_bits: 9 * 128 * 1024 * 4 * 8,
                start_index: 0,
                friendly_name: format!("job {}", i),
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
                info!(
                    "job {}/{}. gpu_id: {}. {}/{} bits.",
                    job.id,
                    job.friendly_name,
                    job.compute_provider_id,
                    job.bits_written,
                    job.size_bits,
                );
                match job.status.try_into().unwrap() {
                    JobStatus::Completed => {
                        info!("🎉 job completed!");
                        completed_jobs += 1;
                        if completed_jobs == total_jobs {
                            break;
                        }
                    }
                    JobStatus::Stopped => {
                        panic!("💥 job stopped due to error")
                    }
                    JobStatus::Queued => {
                        info!("job queued");
                    }
                    JobStatus::Started => {
                        info!("job in progress...");
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
    let _ = env_logger::builder()
        .is_test(false)
        .filter_level(LevelFilter::Info)
        .try_init();

    let path = env::current_dir().unwrap();
    info!("Path: {:?}", path);

    let (mut api_client, guard) = start_server().await;

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
            post_size_bits: 9 * 128 * 1024 * 4 * 8,
            start_index: 0,
            friendly_name: "world's first pos".to_string(),
        })
        .await
        .unwrap()
        .into_inner();

    let job = resp.job.unwrap();
    info!("job info: {}", job);
    job_status_handler(receiver).await;
    delete_pos_files(&vec![job], config.data_dir);
    info!("{}", guard.0.id());
}

async fn job_status_handler(mut receiver: Streaming<JobStatusStreamResponse>) {
    while let Some(res) = receiver.next().await {
        match res {
            Ok(job_status) => {
                let job = job_status.job.unwrap();
                info!(
                    "job id: {}. name: {}. gpu_id: {}. status: {}/{} bits.",
                    job.id,
                    job.friendly_name,
                    job.compute_provider_id,
                    job.bits_written,
                    job.size_bits,
                );
                match job.status.try_into().unwrap() {
                    JobStatus::Completed => {
                        info!("🎉 job {} completed!", job.id);
                        break;
                    }
                    JobStatus::Stopped => {
                        panic!("💥 job {} stopped due to error", job.id)
                    }
                    JobStatus::Queued => {
                        info!("job {} queued", job.id);
                    }
                    JobStatus::Started => {
                        info!("job {} in progress", job.id);
                    }
                }
            }
            Err(e) => panic!("💥 server error {}", e),
        }
    }
}
