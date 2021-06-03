extern crate log;
extern crate pos_api;

use log::info;
use pos_api::api::job::JobStatus;
use pos_api::api::pos_data_service_client::PosDataServiceClient;
use pos_api::api::{Job, JobStatusStreamResponse};
use std::convert::TryInto;
use std::path::Path;
use std::process::{Child, Command};
use std::time::Duration;
use std::{env, fs};
use tokio::time::sleep;
use tokio_stream::StreamExt;
use tonic::transport::Channel;
use tonic::Streaming;

/// Test helper functions and types

pub struct Guard(pub Child);

/// Child process guard
impl Drop for Guard {
    fn drop(&mut self) {
        let pid = self.0.id() as i32;
        match self.0.kill() {
            Err(e) => info!("could not kill child process {}: {}", pid, e),
            Ok(_) => info!("killed guarded process {}", pid),
        }
    }
}

/// Start a pos server and return grpc client for it
pub async fn start_server(use_cpu_provider: bool) -> (PosDataServiceClient<Channel>, Guard) {
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

/// Delete generated pos files for jobs at the provided data dir
pub fn delete_pos_files(jobs: &Vec<Job>, data_dir: String) {
    for job in jobs {
        let file_name = job.file_name();
        let path = Path::new(data_dir.clone().as_str()).join(file_name);
        info!("deleting post file {}...", path.display());
        let _ = fs::remove_file(path).unwrap();
    }
}

/// For some reason rust linter doesn't recognize this helper is called by tests but it does for the other methods in this file
#[allow(dead_code)]
pub async fn job_status_handler(mut receiver: Streaming<JobStatusStreamResponse>) {
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
