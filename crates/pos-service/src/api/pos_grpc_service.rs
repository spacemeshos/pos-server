use crate::pos_api::api::pos_data_service_server::PosDataService;
use crate::server::{
    AbortJob, AddJob, GetAllJobs, GetAllProviders, GetConfig, GetJob, PosServer, SetConfig,
    SubscribeToJobStatuses,
};
use anyhow::Result;
use pos_api::api::{
    AbortJobRequest, AbortJobResponse, AddJobRequest, AddJobResponse, GetAllJobsStatusRequest,
    GetAllJobsStatusResponse, GetConfigRequest, GetConfigResponse, GetJobStatusRequest,
    GetJobStatusResponse, GetProvidersRequest, GetProvidersResponse, Job, JobStatusStreamRequest,
    JobStatusStreamResponse, Provider, SetConfigRequest, SetConfigResponse,
};
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status};
use xactor::*;

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
    async fn get_providers(
        &self,
        _request: Request<GetProvidersRequest>,
    ) -> Result<Response<GetProvidersResponse>, Status> {
        let server = PosServer::from_registry()
            .await
            .map_err(|e| Status::internal(format!("internal error: {}", e)))?;

        let providers: Vec<Provider> = server
            .call(GetAllProviders {})
            .await
            .map_err(|e| Status::internal(format!("internal error: {}", e)))?
            .map_err(|e| Status::internal(format!("internal error: {}", e)))?;

        Ok(Response::new(GetProvidersResponse { providers }))
    }

    async fn set_config(
        &self,
        request: Request<SetConfigRequest>,
    ) -> Result<Response<SetConfigResponse>, Status> {
        let config = request
            .into_inner()
            .config
            .ok_or_else(|| Status::invalid_argument("missing config"))?;

        let server = PosServer::from_registry()
            .await
            .map_err(|e| Status::internal(format!("internal error: {}", e)))?;

        server
            .call(SetConfig(config))
            .await
            .map_err(|e| Status::internal(format!("internal error: {}", e)))?
            .map_err(|e| Status::internal(format!("internal error: {}", e)))?;

        Ok(Response::new(SetConfigResponse {}))
    }

    async fn get_config(
        &self,
        _request: Request<GetConfigRequest>,
    ) -> Result<Response<GetConfigResponse>, Status> {
        let server = PosServer::from_registry()
            .await
            .map_err(|e| Status::internal(format!("internal error: {}", e)))?;

        let config = server
            .call(GetConfig {})
            .await
            .map_err(|e| Status::internal(format!("internal error: {}", e)))?
            .map_err(|e| Status::internal(format!("internal error: {}", e)))?;

        Ok(Response::new(GetConfigResponse {
            config: Some(config),
        }))
    }

    async fn add_job(
        &self,
        request: Request<AddJobRequest>,
    ) -> Result<Response<AddJobResponse>, Status> {
        let add_job_request = request.into_inner();

        let server = PosServer::from_registry()
            .await
            .map_err(|e| Status::internal(format!("internal error: {}", e)))?;

        let job: Job = server
            .call(AddJob(add_job_request))
            .await
            .map_err(|e| Status::internal(format!("internal error: {}", e)))?
            .map_err(|e| Status::internal(format!("internal error: {}", e)))?;

        Ok(Response::new(AddJobResponse { job: Some(job) }))
    }

    async fn get_job_status(
        &self,
        request: Request<GetJobStatusRequest>,
    ) -> Result<Response<GetJobStatusResponse>, Status> {
        let id = request.into_inner().id;

        let server = PosServer::from_registry()
            .await
            .map_err(|e| Status::internal(format!("internal error: {}", e)))?;

        let res: Option<Job> = server
            .call(GetJob(id))
            .await
            .map_err(|e| Status::internal(format!("internal error: {}", e)))?
            .map_err(|e| Status::internal(format!("internal error: {}", e)))?;

        Ok(Response::new(GetJobStatusResponse { job: res }))
    }

    async fn get_all_jobs_statuses(
        &self,
        _request: Request<GetAllJobsStatusRequest>,
    ) -> Result<Response<GetAllJobsStatusResponse>, Status> {
        let server = PosServer::from_registry()
            .await
            .map_err(|e| Status::internal(format!("internal error: {}", e)))?;

        let jobs: Vec<Job> = server
            .call(GetAllJobs {})
            .await
            .map_err(|e| Status::internal(format!("internal error: {}", e)))?
            .map_err(|e| Status::internal(format!("internal error: {}", e)))?;

        Ok(Response::new(GetAllJobsStatusResponse { jobs }))
    }

    async fn abort_job(
        &self,
        request: Request<AbortJobRequest>,
    ) -> Result<Response<AbortJobResponse>, Status> {
        let server = PosServer::from_registry()
            .await
            .map_err(|e| Status::internal(format!("internal error: {}", e)))?;

        server
            .call(AbortJob(request.into_inner()))
            .await
            .map_err(|e| Status::internal(format!("internal error: {}", e)))?
            .map_err(|e| Status::internal(format!("internal error: {}", e)))?;

        Ok(Response::new(AbortJobResponse {}))
    }

    type SubscribeJobStatusStreamStream = ReceiverStream<Result<JobStatusStreamResponse, Status>>;

    async fn subscribe_job_status_stream(
        &self,
        _request: Request<JobStatusStreamRequest>,
    ) -> Result<Response<Self::SubscribeJobStatusStreamStream>, Status> {
        let server = PosServer::from_registry()
            .await
            .map_err(|e| Status::internal(format!("internal error: {}", e)))?;

        let resp: Self::SubscribeJobStatusStreamStream = server
            .call(SubscribeToJobStatuses {})
            .await
            .map_err(|e| Status::internal(format!("internal error: {}", e)))?
            .map_err(|e| Status::internal(format!("internal error: {}", e)))?;

        Ok(Response::new(resp))
    }
}
