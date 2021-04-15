use crate::api::job::JobStatus;
use crate::api::{Job, Provider};
use anyhow::{bail, Result};
use chrono::{DateTime, Local, TimeZone};
use std::convert::TryFrom;
use std::fmt;
use std::fmt::{Display, Formatter};

const COMPUTE_API_CLASS_CPU: u32 = 1;
const COMPUTE_API_CLASS_CUDA: u32 = 2;
const COMPUTE_API_CLASS_VULKAN: u32 = 3;

fn get_provider_class_string(class: u32) -> &'static str {
    match class {
        0 => "UNSPECIFIED",
        COMPUTE_API_CLASS_CPU => "CPU",
        COMPUTE_API_CLASS_CUDA => "CUDA",
        COMPUTE_API_CLASS_VULKAN => "VULKAN",
        _ => "INVALID",
    }
}

impl TryFrom<i32> for JobStatus {
    type Error = ();
    fn try_from(v: i32) -> Result<Self, Self::Error> {
        match v {
            x if x == JobStatus::Started as i32 => Ok(JobStatus::Started),
            x if x == JobStatus::Queued as i32 => Ok(JobStatus::Queued),
            x if x == JobStatus::Stopped as i32 => Ok(JobStatus::Stopped),
            x if x == JobStatus::Completed as i32 => Ok(JobStatus::Completed),
            _ => Err(()),
        }
    }
}

impl Display for JobStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let str = match self {
            JobStatus::Started => "started",
            JobStatus::Queued => "queued",
            JobStatus::Stopped => "stopped",
            JobStatus::Completed => "completed",
        };
        write!(f, "{}", str)
    }
}

impl Job {
    /// Validate job data
    pub fn validate(&self, index_per_compute: u64, label_size: u32) -> Result<()> {
        if label_size != 8 {
            // we only support 8 bit labels
            bail!("only 8 bits label size is supported")
        }
        let min_size = index_per_compute * label_size as u64;
        if self.size_bits < min_size {
            bail!(
                "pos size is too small. Minimum is {}. Requested {}",
                min_size,
                self.size_bits
            )
        }

        let n: f32 = self.size_bits as f32 / 8.0;
        if (n - n.round()).abs() > 0.000001 {
            bail!("only sizes which are multiples of bytes are supported");
        }

        Ok(())
    }
}

impl Display for Provider {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "id: {}. ", self.id)?;
        write!(f, "model: {}. ", self.model)?;
        write!(f, "class: {}", get_provider_class_string(self.class as u32))
    }
}

impl Display for Job {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "id: {}. ", self.id)?;
        write!(f, "name: {}. ", self.friendly_name)?;
        write!(f, "size (bits): {}. ", self.size_bits)?;
        write!(f, "status: {}. ", self.status)?;

        let submitted: DateTime<Local> = Local.timestamp(self.submitted as i64, 0);
        write!(f, "submitted: {}. ", submitted.to_rfc2822())?;

        let started: DateTime<Local> = Local.timestamp(self.started as i64, 0);
        write!(f, "started: {}. ", started.to_rfc2822())?;

        if self.stopped != 0 {
            let stopped: DateTime<Local> = Local.timestamp(self.stopped as i64, 0);
            write!(f, "stopped: {}. ", stopped.to_rfc2822())?;
        }
        write!(f, "bytes written (bits): {}. ", self.bits_written)?;
        write!(f, "client id: {}. ", hex::encode(self.client_id.clone()))?;

        if let Some(err) = self.last_error.as_ref() {
            write!(f, "last Error: {}, {}. ", err.error, err.message)?;
        }

        write!(f, "gpu id: {}. ", self.compute_provider_id)?;
        write!(f, "pow index: {}.", self.proof_of_work_index)
    }
}
