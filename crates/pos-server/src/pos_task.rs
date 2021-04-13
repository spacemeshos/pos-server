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
use std::fs;

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

    // Start a pos data file creation task for a job
    pub(crate) async fn start_task(&mut self, job: &Job) -> Result<Job> {
        if self.providers_pool.is_empty() {
            bail!("unexpected condition: no available providers")
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

        // todo: add task params validation here before starting

        let _handle = task::spawn_blocking(move || {
            let bits_per_cycle =
                task_config.indexes_per_compute_cycle * task_config.bits_per_index as u64;

            let iterations = task_job.size_bits / bits_per_cycle;
            let _last_cycle_bits = task_job.size_bits - (cycles * bits_per_cycle);

            let mut buffer = vec![
                0_u8;
                (task_config.indexes_per_compute_cycle * task_config.bits_per_index as u64)
                    as usize
            ];

            let mut hashes_computed: u64 = 0;
            let mut hashes_per_sec: u64 = 0;

            // for now we create file called <job_id>.bin in the dest data folder
            let path = Path::new(task_config.data_dir.as_str())
                .join(Path::new(format!("{}.bin", task_job.id).as_str()));

            let mut file = match File::create(&path) {
                Ok(file) => file,
                Err(e) => panic!("error creating pos data file {}: {}", path.display(), e),
            };

            let mut file_writer = BufWriter::new(file);

            for i in 0..iterations {
                let start_idx = i * task_config.indexes_per_compute_cycle;
                let end_idx = start_idx + task_config.indexes_per_compute_cycle - 1;

                info!(
                    "job: {}. executing pos iter {} / {} ",
                    task_job.id, i, iterations
                );

                scrypt_positions(
                    task_job.compute_provider_id,
                    task_job.client_id.as_ref(),
                    start_idx,
                    end_idx,
                    task_config.bits_per_index,
                    task_config.salt.as_ref(),
                    0,
                    &mut buffer,
                    task_config.n,
                    task_config.r,
                    task_config.p,
                    &mut hashes_computed as *mut u64,
                    &mut hashes_per_sec as *mut u64,
                );

                if hashes_computed < task_config.indexes_per_compute_cycle {
                    error!("gpu compute error for job: {}. ", task_job.id);

                    task_job.last_error = Some(JobError {
                        error: 500,
                        message: "gpu compute error".to_string(),
                    });
                    task_job.status = JobStatus::Stopped as i32;
                    task_job.stopped = datetime::Instant::now().seconds() as u64;
                    break;
                }

                match file_writer.write_all(&buffer) {
                    Ok(..) => info!("wrote... fix me"),
                    Err(e) => panic!("error writing to pos data file: {} {}", path.display(), e),
                }

                task_job.bits_written += bits_per_cycle;
                PosServer::update_job_status(&task_job);
            }

            // todo: do last cycle (if needed)

            // todo: flush file
            file_writer.flush().expect("failed to flush pos data file");

            if task_job.status == JobStatus::Started as i32 {
                // if task was running and didn't stop due to an error then mark it as complete
                task_job.status = JobStatus::Completed as i32;
                task_job.stopped = datetime::Instant::now().seconds() as u64;
            }

            PosServer::update_job_status(&task_job);
        });

        Ok(res_job)
    }
}
