use crate::server::{PosServer, UpdateJobStatus};
use anyhow::{bail, Result};
use pos_api::api::job::JobStatus;
use pos_api::api::{Job, JobError};
use std::fs::File;
use std::io::prelude::*;
use std::io::BufWriter;
use std::path::Path;
use tokio::task;
use xactor::*;

use pos_compute::scrypt_positions;
use pos_compute::OPTIONS;

impl PosServer {
    /// helper sync function used to update job status via the server service from blocking code
    fn update_job_status(job: &Job) -> Result<()> {
        let task_job = job.clone();
        tokio::spawn(async move {
            PosServer::from_registry()
                .await
                .expect("failed to get server service from registry")
                .call(UpdateJobStatus(task_job))
                .await
                .expect("failed to call server service")
                .expect("UpdateJobStatus error");
        });
        Ok(())
    }

    /// Report a task error to the server service
    fn task_error(job: &mut Job, error: i32, message: String) {
        let err_msg = format!("job {}: {}", job.id, message);
        error!("{}", err_msg);
        job.last_error = Some(JobError {
            error,
            message: err_msg,
        });
        job.status = JobStatus::Stopped as i32;
        job.stopped = datetime::Instant::now().seconds() as u64;
        let _ = PosServer::update_job_status(job);
    }

    /// Start a pos data file creation task for a job
    pub(crate) async fn start_task(&mut self, job: &Job) -> Result<Job> {
        if self.providers_pool.is_empty() {
            error!(
                "unexpected condition: no available provider. can't process job {}",
                job.id
            );
            bail!("no available provider");
        }

        if let Err(e) = job.validate(
            self.config.indexes_per_compute_cycle,
            self.config.bits_per_index,
        ) {
            error!("job won't run. validation failed. {}, {}", job, e);
            return Err(e);
        }

        let provider_id = self.providers_pool.pop().unwrap();
        let mut task_job = job.clone();

        task_job.started = datetime::Instant::now().seconds() as u64;
        task_job.status = JobStatus::Started as i32;
        task_job.compute_provider_id = provider_id;

        self.jobs.insert(job.id, task_job.clone());
        // return job with updated data
        let res_job = task_job.clone();
        let task_config = self.config.clone();

        info!("starting task for job {}...", task_job.id);

        // span a blocking task since the compute lib computation is blocking
        let _handle = task::spawn_blocking(move || {
            let bits_per_cycle =
                task_config.indexes_per_compute_cycle * task_config.bits_per_index as u64;
            let iterations = task_job.size_bits / bits_per_cycle;
            let buff_size = (task_config.indexes_per_compute_cycle
                * task_config.bits_per_index as u64) as usize;
            let mut buffer = vec![0_u8; buff_size];
            let mut hashes_computed: u64 = 0;
            let mut hashes_per_sec: u64 = 0;
            let mut idx_solution: u64 = 0;

            // for now we create file called <job_id>.pos in the dest data folder
            let path = Path::new(task_config.data_dir.as_str())
                .join(Path::new(format!("{}.pos", task_job.id).as_str()));

            let file = match File::create(&path) {
                Ok(file) => file,
                Err(e) => {
                    PosServer::task_error(
                        &mut task_job,
                        501,
                        format!("error {} creating pos data file at {}.", path.display(), e),
                    );
                    return;
                }
            };

            let mut file_writer = BufWriter::new(file);

            for i in 0..iterations {
                let start_idx = i * task_config.indexes_per_compute_cycle;
                let end_idx = start_idx + task_config.indexes_per_compute_cycle - 1;

                info!(
                    "job: {}. executing pos iter {} / {}, provider: {}. start_idx: {}, end_idx: {}",
                    task_job.id,
                    i + 1,
                    iterations,
                    task_job.compute_provider_id,
                    start_idx,
                    end_idx
                );

                scrypt_positions(
                    task_job.compute_provider_id,
                    task_job.client_id.as_ref(),
                    start_idx,
                    end_idx,
                    task_config.bits_per_index,
                    task_config.salt.as_ref(),
                    OPTIONS::ComputeLeaves as u32,
                    &mut buffer,
                    task_config.n,
                    task_config.r,
                    task_config.p,
                    task_config.d.as_ref(),
                    &mut idx_solution as *mut u64,
                    &mut hashes_computed as *mut u64,
                    &mut hashes_per_sec as *mut u64,
                );

                if hashes_computed < task_config.indexes_per_compute_cycle {
                    PosServer::task_error(&mut task_job, 501, "gpu compute error".into());
                    break;
                }

                match file_writer.write_all(&buffer) {
                    Ok(..) => info!(
                        "job {} wrote {} bytes to {}",
                        task_job.id,
                        buff_size,
                        path.display()
                    ),
                    Err(e) => {
                        PosServer::task_error(
                            &mut task_job,
                            501,
                            format!("error writing to pos data file: {} {}", path.display(), e),
                        );
                        break;
                    }
                }

                task_job.bits_written += bits_per_cycle;
                let _ = PosServer::update_job_status(&task_job);
            }

            info!("compute finished {}", task_job.id);

            if let Err(e) = file_writer.flush() {
                PosServer::task_error(
                    &mut task_job,
                    501,
                    format!("error flushing pos file {}. {}.", path.display(), e),
                );
                return;
            }

            if task_job.status == JobStatus::Started as i32 {
                info!("job completed {}", task_job.id);
                // if task was running and didn't stop due to an error then mark it as complete
                task_job.status = JobStatus::Completed as i32;
                task_job.stopped = datetime::Instant::now().seconds() as u64;
            }

            let _ = PosServer::update_job_status(&task_job);
        });

        Ok(res_job)
    }
}
