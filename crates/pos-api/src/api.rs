/// Service configuration
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Config {
    /// The directory where pos data files are created
    /// Path must be accessible by the server
    #[prost(string, tag = "1")]
    pub data_dir: ::prost::alloc::string::String,
    /// number of indexes to compute per gpu compute cycle. e.g. 1024^4
    #[prost(uint64, tag = "2")]
    pub indexes_per_compute_cycle: u64,
    /// should be 8 for now
    #[prost(uint32, tag = "3")]
    pub bits_per_index: u32,
    /// scrypt salt
    #[prost(bytes = "vec", tag = "4")]
    pub salt: ::prost::alloc::vec::Vec<u8>,
    /// scrypt param
    #[prost(uint32, tag = "5")]
    pub n: u32,
    /// scrypt param
    #[prost(uint32, tag = "6")]
    pub r: u32,
    /// scrypt param
    #[prost(uint32, tag = "7")]
    pub p: u32,
}
/// A pos compute provider such as a GPU or a CPU
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Provider {
    #[prost(uint32, tag = "1")]
    pub id: u32,
    #[prost(string, tag = "2")]
    pub model: ::prost::alloc::string::String,
    #[prost(enumeration = "provider::Class", tag = "3")]
    pub class: i32,
}
/// Nested message and enum types in `Provider`.
pub mod provider {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
    #[repr(i32)]
    pub enum Class {
        Cuda = 0,
        Vulkan = 1,
        X86 = 2,
    }
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetProvidersRequest {}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetProvidersResponse {
    #[prost(message, repeated, tag = "1")]
    pub providers: ::prost::alloc::vec::Vec<Provider>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetConfigRequest {}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetConfigResponse {
    #[prost(message, optional, tag = "1")]
    pub config: ::core::option::Option<Config>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SetConfigRequest {
    #[prost(message, optional, tag = "1")]
    pub config: ::core::option::Option<Config>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SetConfigResponse {}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct JobStatusStreamRequest {
    /// pass 0 to get status of all jobs. Set to Job id to receive status just for that job
    #[prost(uint64, tag = "1")]
    pub id: u64,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct JobStatusStreamResponse {
    #[prost(message, optional, tag = "1")]
    pub job: ::core::option::Option<Job>,
}
/// A proof of space data creation job
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Job {
    /// unique job id generated by the service
    #[prost(uint64, tag = "1")]
    pub id: u64,
    /// bits written to the data file (at dir/id.post) (each index 1 byte)
    #[prost(uint64, tag = "2")]
    pub bits_written: u64,
    /// final requested size in bits
    #[prost(uint64, tag = "3")]
    pub size_bits: u64,
    /// time execution started
    #[prost(uint64, tag = "5")]
    pub started: u64,
    /// time submitted
    #[prost(uint64, tag = "6")]
    pub submitted: u64,
    /// time execution completed or stopped due to error
    #[prost(uint64, tag = "7")]
    pub stopped: u64,
    /// job's status
    #[prost(enumeration = "job::JobStatus", tag = "8")]
    pub status: i32,
    /// last error string if job stopped due to an error or empty otherwise
    #[prost(message, optional, tag = "10")]
    pub last_error: ::core::option::Option<JobError>,
    /// client provided friendly name e.g. 'my pos 1'
    #[prost(string, tag = "11")]
    pub friendly_name: ::prost::alloc::string::String,
    /// unique client id (input to pos algorithm)
    #[prost(bytes = "vec", tag = "12")]
    pub client_id: ::prost::alloc::vec::Vec<u8>,
    /// index of the pow solution index. Only available for complete jobs
    #[prost(uint64, tag = "13")]
    pub proof_of_work_index: u64,
    /// compute provider processor id which executed this job - useful for debugging when job fail
    #[prost(uint32, tag = "14")]
    pub compute_provider_id: u32,
}
/// Nested message and enum types in `Job`.
pub mod job {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
    #[repr(i32)]
    pub enum JobStatus {
        /// queued as all provides are busy with other Jobs
        Queued = 0,
        /// started
        Started = 1,
        /// stopped due to an error or user stopped
        Stopped = 2,
        /// Job completed
        Completed = 3,
    }
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct JobError {
    #[prost(enumeration = "job_error::Error", tag = "1")]
    pub error: i32,
    #[prost(string, tag = "2")]
    pub message: ::prost::alloc::string::String,
}
/// Nested message and enum types in `JobError`.
pub mod job_error {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
    #[repr(i32)]
    pub enum Error {
        Unknown = 0,
        IoError = 1,
        GpuComputeError = 2,
    }
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AbortJobRequest {
    /// pass 0 to abort ALL jobs
    #[prost(uint64, tag = "1")]
    pub id: u64,
    /// delete the job from the service
    #[prost(bool, tag = "2")]
    pub delete_job: bool,
    /// delete job data files in store (best effort)
    #[prost(bool, tag = "3")]
    pub delete_data: bool,
}
/// status
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AbortJobResponse {}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetAllJobsStatusRequest {}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetAllJobsStatusResponse {
    #[prost(message, repeated, tag = "1")]
    pub jobs: ::prost::alloc::vec::Vec<Job>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetJobStatusRequest {
    #[prost(uint64, tag = "1")]
    pub id: u64,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetJobStatusResponse {
    #[prost(message, optional, tag = "1")]
    pub job: ::core::option::Option<Job>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AddJobRequest {
    /// unique client id (input to pos algo)
    #[prost(bytes = "vec", tag = "1")]
    pub client_id: ::prost::alloc::vec::Vec<u8>,
    /// requested pos size in bits
    #[prost(uint64, tag = "2")]
    pub post_size_bits: u64,
    /// optional start index - used to continue a stopped job
    #[prost(uint64, tag = "3")]
    pub start_index: u64,
    /// a name set by client to identify the job
    #[prost(string, tag = "4")]
    pub friendly_name: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AddJobResponse {
    #[prost(message, optional, tag = "1")]
    pub job: ::core::option::Option<Job>,
}
#[doc = r" Generated client implementations."]
pub mod pos_data_service_client {
    #![allow(unused_variables, dead_code, missing_docs)]
    use tonic::codegen::*;
    pub struct PosDataServiceClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl PosDataServiceClient<tonic::transport::Channel> {
        #[doc = r" Attempt to create a new client by connecting to a given endpoint."]
        pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
        where
            D: std::convert::TryInto<tonic::transport::Endpoint>,
            D::Error: Into<StdError>,
        {
            let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
            Ok(Self::new(conn))
        }
    }
    impl<T> PosDataServiceClient<T>
    where
        T: tonic::client::GrpcService<tonic::body::BoxBody>,
        T::ResponseBody: Body + HttpBody + Send + 'static,
        T::Error: Into<StdError>,
        <T::ResponseBody as HttpBody>::Error: Into<StdError> + Send,
    {
        pub fn new(inner: T) -> Self {
            let inner = tonic::client::Grpc::new(inner);
            Self { inner }
        }
        pub fn with_interceptor(inner: T, interceptor: impl Into<tonic::Interceptor>) -> Self {
            let inner = tonic::client::Grpc::with_interceptor(inner, interceptor);
            Self { inner }
        }
        pub async fn get_providers(
            &mut self,
            request: impl tonic::IntoRequest<super::GetProvidersRequest>,
        ) -> Result<tonic::Response<super::GetProvidersResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/api.PosDataService/GetProviders");
            self.inner.unary(request.into_request(), path, codec).await
        }
        #[doc = " Set service configuration"]
        #[doc = " Important: Don't set the config while there are jobs running or queued to run."]
        #[doc = " Wait until all jobs have stopped before changing the config."]
        pub async fn set_config(
            &mut self,
            request: impl tonic::IntoRequest<super::SetConfigRequest>,
        ) -> Result<tonic::Response<super::SetConfigResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/api.PosDataService/SetConfig");
            self.inner.unary(request.into_request(), path, codec).await
        }
        #[doc = " Get service current configuration"]
        pub async fn get_config(
            &mut self,
            request: impl tonic::IntoRequest<super::GetConfigRequest>,
        ) -> Result<tonic::Response<super::GetConfigResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/api.PosDataService/GetConfig");
            self.inner.unary(request.into_request(), path, codec).await
        }
        #[doc = " Add a new post job"]
        pub async fn add_job(
            &mut self,
            request: impl tonic::IntoRequest<super::AddJobRequest>,
        ) -> Result<tonic::Response<super::AddJobResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/api.PosDataService/AddJob");
            self.inner.unary(request.into_request(), path, codec).await
        }
        #[doc = " Get current job status"]
        pub async fn get_job_status(
            &mut self,
            request: impl tonic::IntoRequest<super::GetJobStatusRequest>,
        ) -> Result<tonic::Response<super::GetJobStatusResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/api.PosDataService/GetJobStatus");
            self.inner.unary(request.into_request(), path, codec).await
        }
        #[doc = " Get all jobs statuses"]
        pub async fn get_all_jobs_statuses(
            &mut self,
            request: impl tonic::IntoRequest<super::GetAllJobsStatusRequest>,
        ) -> Result<tonic::Response<super::GetAllJobsStatusResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/api.PosDataService/GetAllJobsStatuses");
            self.inner.unary(request.into_request(), path, codec).await
        }
        #[doc = " Abort ajJob or abort all Jobs and optionally delete them"]
        pub async fn abort_job(
            &mut self,
            request: impl tonic::IntoRequest<super::AbortJobRequest>,
        ) -> Result<tonic::Response<super::AbortJobResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/api.PosDataService/AbortJob");
            self.inner.unary(request.into_request(), path, codec).await
        }
        #[doc = " Subscribe to stream of job status updates for a specific job or for all jobs"]
        pub async fn subscribe_job_status_stream(
            &mut self,
            request: impl tonic::IntoRequest<super::JobStatusStreamRequest>,
        ) -> Result<
            tonic::Response<tonic::codec::Streaming<super::JobStatusStreamResponse>>,
            tonic::Status,
        > {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/api.PosDataService/SubscribeJobStatusStream",
            );
            self.inner
                .server_streaming(request.into_request(), path, codec)
                .await
        }
    }
    impl<T: Clone> Clone for PosDataServiceClient<T> {
        fn clone(&self) -> Self {
            Self {
                inner: self.inner.clone(),
            }
        }
    }
    impl<T> std::fmt::Debug for PosDataServiceClient<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "PosDataServiceClient {{ ... }}")
        }
    }
}
#[doc = r" Generated server implementations."]
pub mod pos_data_service_server {
    #![allow(unused_variables, dead_code, missing_docs)]
    use tonic::codegen::*;
    #[doc = "Generated trait containing gRPC methods that should be implemented for use with PosDataServiceServer."]
    #[async_trait]
    pub trait PosDataService: Send + Sync + 'static {
        async fn get_providers(
            &self,
            request: tonic::Request<super::GetProvidersRequest>,
        ) -> Result<tonic::Response<super::GetProvidersResponse>, tonic::Status>;
        #[doc = " Set service configuration"]
        #[doc = " Important: Don't set the config while there are jobs running or queued to run."]
        #[doc = " Wait until all jobs have stopped before changing the config."]
        async fn set_config(
            &self,
            request: tonic::Request<super::SetConfigRequest>,
        ) -> Result<tonic::Response<super::SetConfigResponse>, tonic::Status>;
        #[doc = " Get service current configuration"]
        async fn get_config(
            &self,
            request: tonic::Request<super::GetConfigRequest>,
        ) -> Result<tonic::Response<super::GetConfigResponse>, tonic::Status>;
        #[doc = " Add a new post job"]
        async fn add_job(
            &self,
            request: tonic::Request<super::AddJobRequest>,
        ) -> Result<tonic::Response<super::AddJobResponse>, tonic::Status>;
        #[doc = " Get current job status"]
        async fn get_job_status(
            &self,
            request: tonic::Request<super::GetJobStatusRequest>,
        ) -> Result<tonic::Response<super::GetJobStatusResponse>, tonic::Status>;
        #[doc = " Get all jobs statuses"]
        async fn get_all_jobs_statuses(
            &self,
            request: tonic::Request<super::GetAllJobsStatusRequest>,
        ) -> Result<tonic::Response<super::GetAllJobsStatusResponse>, tonic::Status>;
        #[doc = " Abort ajJob or abort all Jobs and optionally delete them"]
        async fn abort_job(
            &self,
            request: tonic::Request<super::AbortJobRequest>,
        ) -> Result<tonic::Response<super::AbortJobResponse>, tonic::Status>;
        #[doc = "Server streaming response type for the SubscribeJobStatusStream method."]
        type SubscribeJobStatusStreamStream: futures_core::Stream<Item = Result<super::JobStatusStreamResponse, tonic::Status>>
            + Send
            + Sync
            + 'static;
        #[doc = " Subscribe to stream of job status updates for a specific job or for all jobs"]
        async fn subscribe_job_status_stream(
            &self,
            request: tonic::Request<super::JobStatusStreamRequest>,
        ) -> Result<tonic::Response<Self::SubscribeJobStatusStreamStream>, tonic::Status>;
    }
    #[derive(Debug)]
    pub struct PosDataServiceServer<T: PosDataService> {
        inner: _Inner<T>,
    }
    struct _Inner<T>(Arc<T>, Option<tonic::Interceptor>);
    impl<T: PosDataService> PosDataServiceServer<T> {
        pub fn new(inner: T) -> Self {
            let inner = Arc::new(inner);
            let inner = _Inner(inner, None);
            Self { inner }
        }
        pub fn with_interceptor(inner: T, interceptor: impl Into<tonic::Interceptor>) -> Self {
            let inner = Arc::new(inner);
            let inner = _Inner(inner, Some(interceptor.into()));
            Self { inner }
        }
    }
    impl<T, B> Service<http::Request<B>> for PosDataServiceServer<T>
    where
        T: PosDataService,
        B: HttpBody + Send + Sync + 'static,
        B::Error: Into<StdError> + Send + 'static,
    {
        type Response = http::Response<tonic::body::BoxBody>;
        type Error = Never;
        type Future = BoxFuture<Self::Response, Self::Error>;
        fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }
        fn call(&mut self, req: http::Request<B>) -> Self::Future {
            let inner = self.inner.clone();
            match req.uri().path() {
                "/api.PosDataService/GetProviders" => {
                    #[allow(non_camel_case_types)]
                    struct GetProvidersSvc<T: PosDataService>(pub Arc<T>);
                    impl<T: PosDataService> tonic::server::UnaryService<super::GetProvidersRequest>
                        for GetProvidersSvc<T>
                    {
                        type Response = super::GetProvidersResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetProvidersRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).get_providers(request).await };
                            Box::pin(fut)
                        }
                    }
                    let inner = self.inner.clone();
                    let fut = async move {
                        let interceptor = inner.1.clone();
                        let inner = inner.0;
                        let method = GetProvidersSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = if let Some(interceptor) = interceptor {
                            tonic::server::Grpc::with_interceptor(codec, interceptor)
                        } else {
                            tonic::server::Grpc::new(codec)
                        };
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/api.PosDataService/SetConfig" => {
                    #[allow(non_camel_case_types)]
                    struct SetConfigSvc<T: PosDataService>(pub Arc<T>);
                    impl<T: PosDataService> tonic::server::UnaryService<super::SetConfigRequest> for SetConfigSvc<T> {
                        type Response = super::SetConfigResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::SetConfigRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).set_config(request).await };
                            Box::pin(fut)
                        }
                    }
                    let inner = self.inner.clone();
                    let fut = async move {
                        let interceptor = inner.1.clone();
                        let inner = inner.0;
                        let method = SetConfigSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = if let Some(interceptor) = interceptor {
                            tonic::server::Grpc::with_interceptor(codec, interceptor)
                        } else {
                            tonic::server::Grpc::new(codec)
                        };
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/api.PosDataService/GetConfig" => {
                    #[allow(non_camel_case_types)]
                    struct GetConfigSvc<T: PosDataService>(pub Arc<T>);
                    impl<T: PosDataService> tonic::server::UnaryService<super::GetConfigRequest> for GetConfigSvc<T> {
                        type Response = super::GetConfigResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetConfigRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).get_config(request).await };
                            Box::pin(fut)
                        }
                    }
                    let inner = self.inner.clone();
                    let fut = async move {
                        let interceptor = inner.1.clone();
                        let inner = inner.0;
                        let method = GetConfigSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = if let Some(interceptor) = interceptor {
                            tonic::server::Grpc::with_interceptor(codec, interceptor)
                        } else {
                            tonic::server::Grpc::new(codec)
                        };
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/api.PosDataService/AddJob" => {
                    #[allow(non_camel_case_types)]
                    struct AddJobSvc<T: PosDataService>(pub Arc<T>);
                    impl<T: PosDataService> tonic::server::UnaryService<super::AddJobRequest> for AddJobSvc<T> {
                        type Response = super::AddJobResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::AddJobRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).add_job(request).await };
                            Box::pin(fut)
                        }
                    }
                    let inner = self.inner.clone();
                    let fut = async move {
                        let interceptor = inner.1.clone();
                        let inner = inner.0;
                        let method = AddJobSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = if let Some(interceptor) = interceptor {
                            tonic::server::Grpc::with_interceptor(codec, interceptor)
                        } else {
                            tonic::server::Grpc::new(codec)
                        };
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/api.PosDataService/GetJobStatus" => {
                    #[allow(non_camel_case_types)]
                    struct GetJobStatusSvc<T: PosDataService>(pub Arc<T>);
                    impl<T: PosDataService> tonic::server::UnaryService<super::GetJobStatusRequest>
                        for GetJobStatusSvc<T>
                    {
                        type Response = super::GetJobStatusResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetJobStatusRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).get_job_status(request).await };
                            Box::pin(fut)
                        }
                    }
                    let inner = self.inner.clone();
                    let fut = async move {
                        let interceptor = inner.1.clone();
                        let inner = inner.0;
                        let method = GetJobStatusSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = if let Some(interceptor) = interceptor {
                            tonic::server::Grpc::with_interceptor(codec, interceptor)
                        } else {
                            tonic::server::Grpc::new(codec)
                        };
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/api.PosDataService/GetAllJobsStatuses" => {
                    #[allow(non_camel_case_types)]
                    struct GetAllJobsStatusesSvc<T: PosDataService>(pub Arc<T>);
                    impl<T: PosDataService>
                        tonic::server::UnaryService<super::GetAllJobsStatusRequest>
                        for GetAllJobsStatusesSvc<T>
                    {
                        type Response = super::GetAllJobsStatusResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetAllJobsStatusRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).get_all_jobs_statuses(request).await };
                            Box::pin(fut)
                        }
                    }
                    let inner = self.inner.clone();
                    let fut = async move {
                        let interceptor = inner.1.clone();
                        let inner = inner.0;
                        let method = GetAllJobsStatusesSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = if let Some(interceptor) = interceptor {
                            tonic::server::Grpc::with_interceptor(codec, interceptor)
                        } else {
                            tonic::server::Grpc::new(codec)
                        };
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/api.PosDataService/AbortJob" => {
                    #[allow(non_camel_case_types)]
                    struct AbortJobSvc<T: PosDataService>(pub Arc<T>);
                    impl<T: PosDataService> tonic::server::UnaryService<super::AbortJobRequest> for AbortJobSvc<T> {
                        type Response = super::AbortJobResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::AbortJobRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).abort_job(request).await };
                            Box::pin(fut)
                        }
                    }
                    let inner = self.inner.clone();
                    let fut = async move {
                        let interceptor = inner.1.clone();
                        let inner = inner.0;
                        let method = AbortJobSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = if let Some(interceptor) = interceptor {
                            tonic::server::Grpc::with_interceptor(codec, interceptor)
                        } else {
                            tonic::server::Grpc::new(codec)
                        };
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/api.PosDataService/SubscribeJobStatusStream" => {
                    #[allow(non_camel_case_types)]
                    struct SubscribeJobStatusStreamSvc<T: PosDataService>(pub Arc<T>);
                    impl<T: PosDataService>
                        tonic::server::ServerStreamingService<super::JobStatusStreamRequest>
                        for SubscribeJobStatusStreamSvc<T>
                    {
                        type Response = super::JobStatusStreamResponse;
                        type ResponseStream = T::SubscribeJobStatusStreamStream;
                        type Future =
                            BoxFuture<tonic::Response<Self::ResponseStream>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::JobStatusStreamRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut =
                                async move { (*inner).subscribe_job_status_stream(request).await };
                            Box::pin(fut)
                        }
                    }
                    let inner = self.inner.clone();
                    let fut = async move {
                        let interceptor = inner.1;
                        let inner = inner.0;
                        let method = SubscribeJobStatusStreamSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = if let Some(interceptor) = interceptor {
                            tonic::server::Grpc::with_interceptor(codec, interceptor)
                        } else {
                            tonic::server::Grpc::new(codec)
                        };
                        let res = grpc.server_streaming(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                _ => Box::pin(async move {
                    Ok(http::Response::builder()
                        .status(200)
                        .header("grpc-status", "12")
                        .header("content-type", "application/grpc")
                        .body(tonic::body::BoxBody::empty())
                        .unwrap())
                }),
            }
        }
    }
    impl<T: PosDataService> Clone for PosDataServiceServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self { inner }
        }
    }
    impl<T: PosDataService> Clone for _Inner<T> {
        fn clone(&self) -> Self {
            Self(self.0.clone(), self.1.clone())
        }
    }
    impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }
    impl<T: PosDataService> tonic::transport::NamedService for PosDataServiceServer<T> {
        const NAME: &'static str = "api.PosDataService";
    }
}
