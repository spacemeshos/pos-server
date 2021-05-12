use crate::api::job::JobStatus;
use crate::api::{Job, Provider};
use anyhow::{bail, Result};
use chrono::{DateTime, Local, TimeZone};
use std::convert::TryFrom;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::ops::Sub;

// Compute operation result
pub enum ComputeResults {
    NoError = 0,
    PowSolutionFound = 1,
    ComputeError = -1,
    Timeout = -2,
    Already = -3,
    Canceled = -4,
    MissingComputeOptions = -5,
    InvalidParam = -6,
}

impl TryFrom<i32> for ComputeResults {
    type Error = ();
    fn try_from(v: i32) -> Result<Self, Self::Error> {
        match v {
            x if x == ComputeResults::NoError as i32 => Ok(ComputeResults::NoError),
            x if x == ComputeResults::PowSolutionFound as i32 => {
                Ok(ComputeResults::PowSolutionFound)
            }
            x if x == ComputeResults::ComputeError as i32 => Ok(ComputeResults::ComputeError),
            x if x == ComputeResults::Timeout as i32 => Ok(ComputeResults::Timeout),
            x if x == ComputeResults::Already as i32 => Ok(ComputeResults::Already),
            x if x == ComputeResults::Canceled as i32 => Ok(ComputeResults::Canceled),
            x if x == ComputeResults::MissingComputeOptions as i32 => {
                Ok(ComputeResults::MissingComputeOptions)
            }
            x if x == ComputeResults::InvalidParam as i32 => Ok(ComputeResults::InvalidParam),

            _ => Err(()),
        }
    }
}

impl Display for ComputeResults {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let str = match self {
            ComputeResults::NoError => "No error",
            ComputeResults::PowSolutionFound => "Pow solution found",
            ComputeResults::ComputeError => "Compute error",
            ComputeResults::Timeout => "Timeout",
            ComputeResults::Already => "Already",
            ComputeResults::Canceled => "Canceled",
            ComputeResults::MissingComputeOptions => "Missing options",
            ComputeResults::InvalidParam => "Invalid params",
        };
        write!(f, "{}", str)
    }
}

pub enum ComputeOptions {
    ComputeLeaves = 1,
    ComputePow = 2,
}

pub enum ComputeClass {
    Cpu = 1,
    Cuda = 2,
    Vulkan = 3,
}

pub fn get_provider_class_string(class: u32) -> &'static str {
    match class {
        x if x == ComputeClass::Cpu as u32 => "CPU",
        x if x == ComputeClass::Cuda as u32 => "CUDA",
        x if x == ComputeClass::Vulkan as u32 => "VULKAN",
        _ => "UNSPECIFIED",
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
    pub fn file_name(&self) -> String {
        format!("{}.pos", self.id)
    }

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
        let status = JobStatus::try_from(self.status).unwrap();
        write!(f, "status: {}. ", status)?;

        let submitted: DateTime<Local> = Local.timestamp(self.submitted as i64, 0);
        write!(f, "submitted: {}. ", submitted.to_rfc2822())?;

        let started: DateTime<Local> = Local.timestamp(self.started as i64, 0);
        write!(f, "started: {}. ", started.to_rfc2822())?;

        let stopped: DateTime<Local> = Local.timestamp(self.stopped as i64, 0);
        if self.stopped != 0 {
            write!(f, "stopped: {}. ", stopped.to_rfc2822())?;
        }

        if self.status == JobStatus::Completed as i32 {
            let time = stopped.sub(started);
            if time.num_seconds() == 0 {
                write!(f, "Completed in less than 1 second.",)?;
            } else {
                let bytes_per_sec = self.size_bits / (time.num_seconds() as u64 * 8);
                write!(
                    f,
                    "Completed. work duration: {} Secs. Bytes/sec: {}. ",
                    time.num_seconds(),
                    bytes_per_sec
                )?;
            }
        }

        write!(f, "data written (bits): {}. ", self.bits_written)?;
        write!(f, "client id: {}. ", hex::encode(self.client_id.clone()))?;

        if let Some(err) = self.last_error.as_ref() {
            write!(f, "last Error: {}, {}. ", err.error, err.message)?;
        }

        write!(f, "gpu id: {}. ", self.compute_provider_id)?;
        write!(f, "pow index: {}.", self.proof_of_work_index)
    }
}
