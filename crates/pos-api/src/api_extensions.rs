use crate::api::job::JobStatus;
use crate::api::Job;
use chrono::{DateTime, Local, TimeZone};
use std::fmt;
use std::fmt::{Display, Formatter};

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

impl Display for Job {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "id: {}", self.id)?;
        write!(f, "name: {}", self.friendly_name)?;
        write!(f, "size (bits): {}", self.size_bits)?;
        write!(f, "status: {}", self.status)?;

        let submitted: DateTime<Local> = Local.timestamp(self.submitted as i64, 0);
        write!(f, "submitted: {}", submitted.to_rfc2822())?;

        let started: DateTime<Local> = Local.timestamp(self.started as i64, 0);
        write!(f, "started: {}", started.to_rfc2822())?;

        if self.stopped != 0 {
            let stopped: DateTime<Local> = Local.timestamp(self.stopped as i64, 0);
            write!(f, "stopped: {}", stopped.to_rfc2822())?;
        }
        write!(f, "bytes written (bits): {}", self.bits_written)?;

        write!(f, "client id: {}", hex::encode(self.client_id.clone()))?;

        if let Some(err) = self.last_error.as_ref() {
            write!(f, "last Error: {}, {}", err.error, err.message)?;
        }

        write!(f, "gpu id: {}", self.compute_provider_id)?;

        write!(f, "pow index: {}", self.proof_of_work_index)
    }
}
