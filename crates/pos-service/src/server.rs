use crate::api::pos_grpc_service::PosGrpcService;
use crate::{DEFAULT_BITS_PER_INDEX, DEFAULT_INDEXED_PER_CYCLE, DEFAULT_SALT};
use anyhow::Result;
use pos_api::api::job::JobStatus;
use pos_api::api::pos_data_service_server::PosDataServiceServer;
use pos_api::api::{
    AbortJobRequest, AddJobRequest, Config, Job, JobStatusStreamResponse, Provider,
};
use pos_compute::{get_providers, PosComputeProvider, COMPUTE_API_CLASS_CPU};
use std::collections::HashMap;
use tokio::sync::mpsc;
use tokio::sync::mpsc::Sender;
use tokio_stream::wrappers::ReceiverStream;
use tonic::transport::Server;
use tonic::Status;
use xactor::*;

/// PosServer is a Spacemesh proof of space data generator service.
/// The service manages a pool of compute providers (gpus) and schedules
/// client-submitted jobs to use these providers to create pos data and to report job
/// progress and errors to clients.
/// todo: support aborting an in-progress job
pub(crate) struct PosServer {
    providers: Vec<PosComputeProvider>,  // gpu compute providers
    pending_jobs: Vec<Job>,              // pending
    pub(crate) jobs: HashMap<u64, Job>,  // in progress
    pub(crate) config: Config,           // compute config
    pub(crate) providers_pool: Vec<u32>, // idle providers
    job_status_subscribers: HashMap<u64, Sender<Result<JobStatusStreamResponse, Status>>>,
}

#[async_trait::async_trait]
impl Actor for PosServer {
    async fn started(&mut self, _ctx: &mut Context<Self>) -> Result<()> {
        info!("PosServer system service starting...");
        Ok(())
    }

    async fn stopped(&mut self, _ctx: &mut Context<Self>) {
        info!("PosServer system service stopped");
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
                data_dir: "./".to_string(),
                indexes_per_compute_cycle: DEFAULT_INDEXED_PER_CYCLE,
                bits_per_index: DEFAULT_BITS_PER_INDEX,
                salt: hex::decode(DEFAULT_SALT).unwrap(),
                n: 512,
                r: 1,
                p: 1,
            },
            providers_pool: vec![],
            job_status_subscribers: HashMap::default(),
        }
    }
}

#[message(result = "Result<()>")]
pub(crate) struct Init {
    /// server base config - must be set when initializing
    pub(crate) use_cpu_providers: bool,
}

/// Init the service
#[async_trait::async_trait]
impl Handler<Init> for PosServer {
    async fn handle(&mut self, _ctx: &mut Context<Self>, msg: Init) -> Result<()> {
        for p in get_providers() {
            if !msg.use_cpu_providers && p.compute_api == COMPUTE_API_CLASS_CPU {
                info!(
                    "skipping cpu provider id: {}, model: {}, compute_api: {}",
                    p.id,
                    p.model,
                    pos_api::api_extensions::get_provider_class_string(p.compute_api)
                );
                continue;
            }
            info!(
                "Adding to pool provider id: {}, model: {}, compute_api: {}",
                p.id,
                p.model,
                pos_api::api_extensions::get_provider_class_string(p.compute_api)
            );
            self.providers_pool.push(p.id);
            self.providers.push(p);
        }
        Ok(())
    }
}

#[message(result = "Result<Vec<Provider>>")]
pub(crate) struct GetAllProviders;

// Returns all system providers
#[async_trait::async_trait]
impl Handler<GetAllProviders> for PosServer {
    async fn handle(
        &mut self,
        _ctx: &mut Context<Self>,
        _msg: GetAllProviders,
    ) -> Result<Vec<Provider>> {
        let mut res = vec![];
        for p in self.providers.iter() {
            res.push(Provider {
                id: p.id,
                model: p.model.clone(),
                class: p.compute_api as i32,
            })
        }
        Ok(res)
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
                info!(
                    "job {} finished. Releasing gpu {} pool",
                    updated_job.id, updated_job.compute_provider_id
                );
                // Job stopped or completed - release provider id of job to pool
                self.providers_pool.push(updated_job.compute_provider_id);

                // pick a pending job any start it
                if let Some(new_job) = self.pending_jobs.pop() {
                    info!("starting queued job {}", new_job.id);
                    self.start_task(&new_job).await?;
                } else {
                    info!("no queued jobs");
                }
            }
            // update job data
            self.jobs.insert(updated_job.id, updated_job.clone());
        } else if let Some(idx) = self
            .pending_jobs
            .iter()
            .position(|j| j.id == updated_job.id)
        {
            self.pending_jobs.remove(idx);
            self.pending_jobs.insert(idx, updated_job.clone());
        } else {
            error!("unrecognized job")
        }

        // update all job status subscribers
        for sub in self.job_status_subscribers.clone().iter() {
            let res = sub
                .1
                .send(Ok(JobStatusStreamResponse {
                    job: Some(updated_job.clone()),
                }))
                .await;

            match res {
                Ok(()) => info!("sent updated job status to subscriber"),
                Err(e) => {
                    error!(
                        "failed to send updated job status to subscriber. deleting it: {}",
                        e
                    );
                    self.job_status_subscribers.remove(sub.0);
                }
            }
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

        let job = Job {
            id: rand::random(),
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

        if let Err(e) = job.validate(
            self.config.indexes_per_compute_cycle,
            self.config.bits_per_index,
        ) {
            error!("job can't be added - validation failed: {}, {}", job, e);
            return Err(e);
        }

        if self.providers_pool.is_empty() {
            // all providers busy with in-progress jobs - queue the job
            self.pending_jobs.push(job.clone());
            info!("all providers are busy - queueing job {}...", job.id);
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

/////////////////////////////////////////////

#[message(result = "Result<ReceiverStream<Result<JobStatusStreamResponse, Status>>>")]
pub(crate) struct SubscribeToJobStatuses {}

#[async_trait::async_trait]
impl Handler<SubscribeToJobStatuses> for PosServer {
    async fn handle(
        &mut self,
        _ctx: &mut Context<Self>,
        _msg: SubscribeToJobStatuses,
    ) -> Result<ReceiverStream<Result<JobStatusStreamResponse, Status>>> {
        // create channel for streaming job statuses
        let (tx, rx) = mpsc::channel(32);

        // store the sender indexed by a new unique id
        self.job_status_subscribers.insert(rand::random(), tx);

        // return the receiver
        Ok(ReceiverStream::new(rx))
    }
}

/////////////////////////////////////////////

#[message(result = "Result<()>")]
pub(crate) struct StartGrpcService {
    pub(crate) port: u32,
    pub(crate) host: String,
}
#[async_trait::async_trait]
impl Handler<StartGrpcService> for PosServer {
    async fn handle(&mut self, _ctx: &mut Context<Self>, msg: StartGrpcService) -> Result<()> {
        let addr = format!("{}:{}", msg.host, msg.port).parse().unwrap();
        info!("starting grpc service on: {}...", addr);

        // todo: add a grpc health service
        tokio::task::spawn(async move {
            let res = Server::builder()
                .add_service(PosDataServiceServer::new(PosGrpcService::default()))
                .serve(addr)
                .await;
            if res.is_err() {
                panic!("grpc server stopped due to error: {:?}", res.err().unwrap());
            } else {
                info!("grpc server stopped");
            }
        });

        Ok(())
    }
}
