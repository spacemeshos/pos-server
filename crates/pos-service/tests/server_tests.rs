#[macro_use]
extern crate log;
extern crate nix;

extern crate pos_api;

use log::LevelFilter;
use nix::sys::signal;
use nix::sys::signal::Signal;
use nix::unistd::Pid;
use std::env;
use std::process::{Child, Command};
use std::time::Duration;
use tokio::time::sleep;

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

use pos_api::api::job::JobStatus;
use pos_api::api::pos_data_service_client::PosDataServiceClient;
use pos_api::api::{AddJobRequest, GetConfigRequest, GetProvidersRequest, JobStatusStreamRequest};
use std::convert::TryInto;
use tokio_stream::StreamExt;

#[tokio::test]
async fn simple_job_test() {
    let _ = env_logger::builder()
        .is_test(false)
        .filter_level(LevelFilter::Info)
        .try_init();

    let path = env::current_dir().unwrap();
    info!("Path: {:?}", path);

    let server_path = "../../target/debug/pos-service";
    let server = Command::new(server_path).spawn().unwrap();
    let guard = Guard(server);

    // grpc warmup
    sleep(Duration::from_secs(5)).await;

    let client_id = hex::decode("1215eda121").unwrap();
    let mut api_client = PosDataServiceClient::connect(format!("http://[::1]:{}", 6667))
        .await
        .expect("failed to connect to grpc server");

    let _config = api_client
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
    let mut receiver = api_client
        .subscribe_job_status_stream(JobStatusStreamRequest { id: 0 })
        .await
        .unwrap()
        .into_inner();

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

    while let Some(res) = receiver.next().await {
        match res {
            Ok(job_status) => {
                let job = job_status.job.unwrap();
                info!(
                    "job {}/{}. {} / {} ",
                    job.id, job.friendly_name, job.bits_written, job.size_bits,
                );
                match job.status.try_into().unwrap() {
                    JobStatus::Completed => {
                        info!("🎉 job completed!");
                        break;
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

    info!("{}", guard.0.id());
}
