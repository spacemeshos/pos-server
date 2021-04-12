use crate::pos_api::api::pos_data_service_server::PosDataService;
use anyhow::Result;
use pos_api::api::{
    AbortJobRequest, AbortJobResponse, AddJobRequest, AddJobResponse, GetAllJobsStatusRequest,
    GetAllJobsStatusResponse, GetConfigRequest, GetConfigResponse, GetJobStatusRequest,
    GetJobStatusResponse, SetConfigRequest, SetConfigResponse,
};
// use futures::Stream;
// use tokio::sync::mpsc;

use tonic::{Request, Response, Status};

#[derive(Debug)]
pub(crate) struct PosGrpcService {}

impl Default for PosGrpcService {
    fn default() -> Self {
        debug!("PosService grpc service started");
        PosGrpcService {}
    }
}

impl PosGrpcService {}

#[tonic::async_trait]
impl PosDataService for PosGrpcService {
    async fn set_config(
        &self,
        _request: Request<SetConfigRequest>,
    ) -> Result<Response<SetConfigResponse>, Status> {
        todo!()
    }

    async fn get_config(
        &self,
        _request: Request<GetConfigRequest>,
    ) -> Result<Response<GetConfigResponse>, Status> {
        todo!()
    }

    async fn add_job(
        &self,
        _request: Request<AddJobRequest>,
    ) -> Result<Response<AddJobResponse>, Status> {
        todo!()
    }

    async fn get_job_status(
        &self,
        _request: Request<GetJobStatusRequest>,
    ) -> Result<Response<GetJobStatusResponse>, Status> {
        todo!()
    }

    async fn get_all_jobs_statuses(
        &self,
        _request: Request<GetAllJobsStatusRequest>,
    ) -> Result<Response<GetAllJobsStatusResponse>, Status> {
        todo!()
    }

    async fn abort_job(
        &self,
        _request: Request<AbortJobRequest>,
    ) -> Result<Response<AbortJobResponse>, Status> {
        todo!()
    }
}
