use crate::api::pos_grpc_service::PosGrpcService;

use anyhow::bail;
use anyhow::Result;
use datetime::Instant;
use pos_api::api::job::JobStatus;
use pos_api::api::pos_data_service_server::PosDataServiceServer;
use pos_api::api::{AbortJobRequest, AddJobRequest, Config, Job, JobError};
use pos_compute::{get_providers, PosComputeProvider};
use rand_core::{OsRng, RngCore};
use std::collections::HashMap;
use tonic::transport::Server;
use xactor::*;

pub(crate) struct PosServer {
    providers: Vec<PosComputeProvider>,  // gpu compute providers
    pending_jobs: Vec<Job>,              // pending
    pub(crate) jobs: HashMap<u64, Job>,  // in progress
    pub(crate) config: Config,           // compute config
    pub(crate) providers_pool: Vec<u32>, // idle providers
}

#[async_trait::async_trait]
impl Actor for PosServer {
    async fn started(&mut self, _ctx: &mut Context<Self>) -> Result<()> {
        debug!("PosServer system service starting...");
        Ok(())
    }

    async fn stopped(&mut self, _ctx: &mut Context<Self>) {
        debug!("PosServer system service stopped");
    }
}

impl Service for PosServer {}
impl Default for PosServer {
    fn default() -> Self {
        PosServer {
            providers: vec![],
            pending_jobs: vec![],
            jobs: Default::default(),
            config: Config {
                // default config
                data_dir: "./".to_string(),
                indexes_per_compute_cycle: 9 * 128 * 1024,
                bits_per_index: 8,
                salt: vec![],
                n: 512,
                r: 1,
                p: 1,
            },
            providers_pool: vec![],
        }
    }
}
#[message(result = "Result<()>")]
pub(crate) struct Init {}

/// Init the service
#[async_trait::async_trait]
impl Handler<Init> for PosServer {
    async fn handle(&mut self, _ctx: &mut Context<Self>, _msg: Init) -> Result<()> {
        self.providers = get_providers();
        for provider in self.providers.iter() {
            self.providers_pool.push(provider.id)
        }
        Ok(())
    }
}

#[message(result = "Result<Vec<Job>>")]
pub(crate) struct GetAllJobs;

// Returns job with current status for job id
#[async_trait::async_trait]
impl Handler<GetAllJobs> for PosServer {
    async fn handle(&mut self, _ctx: &mut Context<Self>, _msg: GetAllJobs) -> Result<Vec<Job>> {
        let mut res: Vec<Job> = self.jobs.values().cloned().collect();
        for job in self.pending_jobs.iter() {
            res.push(job.clone())
        }
        Ok(res)
    }
}

#[message(result = "Result<Option<Job>>")]
pub(crate) struct GetJob(pub(crate) u64);

// Returns job with current status for job id
#[async_trait::async_trait]
impl Handler<GetJob> for PosServer {
    async fn handle(&mut self, _ctx: &mut Context<Self>, msg: GetJob) -> Result<Option<Job>> {
        if let Some(job) = self.jobs.get(&msg.0) {
            Ok(Some(job.clone()))
        } else if let Some(job) = self.pending_jobs.iter().find(|&j| j.id == msg.0) {
            Ok(Some(job.clone()))
        } else {
            Ok(None)
        }
    }
}

#[message(result = "Result<()>")]
pub(crate) struct UpdateJobStatus(pub(crate) Job);

// Update job status - should only be called from a task which is processing the job
#[async_trait::async_trait]
impl Handler<UpdateJobStatus> for PosServer {
    async fn handle(&mut self, _ctx: &mut Context<Self>, msg: UpdateJobStatus) -> Result<()> {
        let updated_job = msg.0;
        if let Some(_) = self.jobs.get(&updated_job.id) {
            // job is running or stopped

            if updated_job.status != JobStatus::Started as i32 {
                // Job stopped or completed - release provider id of job to pool
                self.providers_pool.push(updated_job.compute_provider_id);
            }
            // update job data
            self.jobs.insert(updated_job.id, updated_job);
        } else if let Some(idx) = self
            .pending_jobs
            .iter()
            .position(|j| j.id == updated_job.id)
        {
            self.pending_jobs.remove(idx);
            self.pending_jobs.insert(idx, updated_job);
        } else {
            error!("unrecognized job")
        }
        Ok(())
    }
}

#[message(result = "Result<Job>")]
pub(crate) struct AddJob(pub(crate) AddJobRequest);

/// Set the pos compute config
#[async_trait::async_trait]
impl Handler<AddJob> for PosServer {
    async fn handle(&mut self, _ctx: &mut Context<Self>, msg: AddJob) -> Result<Job> {
        let data = msg.0;

        // todo: verify data.post_size is in powers of 2 + below max

        let mut job = Job {
            id: OsRng.next_u64(),
            bits_written: 0,
            size_bits: data.post_size_bits,
            started: 0,
            submitted: datetime::Instant::now().seconds() as u64,
            stopped: 0,
            status: JobStatus::Queued as i32,
            last_error: None,
            friendly_name: data.friendly_name,
            client_id: data.client_id,
            proof_of_work_index: u64::MAX,
            compute_provider_id: u32::MAX,
        };

        if self.providers_pool.is_empty() {
            // all providers busy with in-progress jobs - queue the job
            self.pending_jobs.push(job.clone());
            info!("all providers are busy - queueing job");
            return Ok(job);
        }

        let res_job = self.start_task(&job).await?;
        Ok(res_job)
    }
}

#[message(result = "Result<(Config)>")]
pub(crate) struct GetConfig;

/// Get the current pos compute config
#[async_trait::async_trait]
impl Handler<GetConfig> for PosServer {
    async fn handle(&mut self, _ctx: &mut Context<Self>, _msg: GetConfig) -> Result<Config> {
        Ok(self.config.clone())
    }
}

#[message(result = "Result<()>")]
pub(crate) struct AbortJob(pub(crate) AbortJobRequest);

/// Set the pos compute config
#[async_trait::async_trait]
impl Handler<AbortJob> for PosServer {
    async fn handle(&mut self, _ctx: &mut Context<Self>, msg: AbortJob) -> Result<()> {
        let req = msg.0;

        if let Some(_job) = self.jobs.get(&req.id) {
            // todo: abort on-going job - need to do this via sending a message the blocking task

            if req.delete_data {
                // todo: attempt to delete all job files in store (best effort)
            }
        }

        if req.delete_job {
            // remove job

            if let Some(idx) = self.pending_jobs.iter().position(|j| j.id == req.id) {
                self.pending_jobs.remove(idx);
            }
            self.jobs.remove(&req.id);
        }

        Ok(())
    }
}

#[message(result = "Result<()>")]
pub(crate) struct SetConfig(pub(crate) Config);

/// Set the pos compute config
#[async_trait::async_trait]
impl Handler<SetConfig> for PosServer {
    async fn handle(&mut self, _ctx: &mut Context<Self>, msg: SetConfig) -> Result<()> {
        self.config = msg.0;
        Ok(())
    }
}

#[message(result = "Result<()>")]
pub(crate) struct StartGrpcService {
    pub(crate) port: u32,
    pub(crate) host: String,
}
#[async_trait::async_trait]
impl Handler<StartGrpcService> for PosServer {
    async fn handle(&mut self, _ctx: &mut Context<Self>, msg: StartGrpcService) -> Result<()> {
        let addr = format!("{}:{}", msg.host, msg.port).parse().unwrap();

        info!("starting grpc service on: {}", addr);

        tokio::task::spawn(async move {
            let res = Server::builder()
                .add_service(PosDataServiceServer::new(PosGrpcService::default()))
                .serve(addr)
                .await;

            if res.is_err() {
                info!("grpc server stopped. {:?}", res.err().unwrap());
            } else {
                info!("grpc server stopped");
            }
        });

        Ok(())
    }
}
