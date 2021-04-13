use crate::api::pos_grpc_service::PosGrpcService;

use anyhow::Result;
use pos_api::api::pos_data_service_server::PosDataServiceServer;
use pos_api::api::{Config, Job};
use pos_compute::{get_providers, PosComputeProvider};
use std::collections::HashMap;
use tonic::transport::Server;
use xactor::*;

pub(crate) struct PosServer {
    providers: Vec<PosComputeProvider>, // gpu compute providers
    pending_jobs: Vec<Job>,             // pending
    jobs: HashMap<u64, Job>,            // in progress
    config: Config,                     // compute config
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
            },
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
pub(crate) struct GetJob(u64);

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
pub(crate) struct UpdateJobStatus(Job);

// Update job status - should only be called from a task which is processing the job
#[async_trait::async_trait]
impl Handler<UpdateJobStatus> for PosServer {
    async fn handle(&mut self, _ctx: &mut Context<Self>, msg: UpdateJobStatus) -> Result<()> {
        let updated_job = msg.0;
        if let Some(_) = self.jobs.get(&updated_job.id) {
            // job is in progress - replace status with updated status
            self.jobs.insert(updated_job.id, updated_job);
        } else if let Some(idx) = self
            .pending_jobs
            .iter()
            .position(|&j| j.id == updated_job.id)
        {
            self.pending_jobs.remove(idx);
            self.pending_jobs.insert(idx, updated_job);
        } else {
            error!("unrecognized job")
        }
        Ok(())
    }
}

#[message(result = "Result<()>")]
pub(crate) struct AddJob(Job);

/// Set the pos compute config
#[async_trait::async_trait]
impl Handler<AddJob> for PosServer {
    async fn handle(&mut self, _ctx: &mut Context<Self>, msg: AddJob) -> Result<()> {
        let job = msg.0;
        if self.providers.len() == self.jobs.len() {
            // all providers busy with in-progress jobs - queue the job
            self.pending_jobs.push(job);
            info!("all providers are busy - queueing job");
            return Ok(());
        }

        let _task_job = job.clone();
        self.jobs.insert(job.id, job);

        // todo: start the job's task here - using tokio blocking spwan task - pass a copy of the job to the task
        // task uses this system service to call back on progress

        Ok(())
    }
}

#[message(result = "Result<(Config)>")]
pub(crate) struct GetConfig(Config);

/// Get the current pos compute config
#[async_trait::async_trait]
impl Handler<GetConfig> for PosServer {
    async fn handle(&mut self, _ctx: &mut Context<Self>, _msg: GetConfig) -> Result<Config> {
        Ok(self.config.clone())
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
