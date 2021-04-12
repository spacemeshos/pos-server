use crate::api::pos_grpc_service::PosGrpcService;
use anyhow::Result;
use pos_api::api::pos_data_service_server::PosDataServiceServer;
use tonic::transport::Server;
use xactor::*;

pub(crate) struct PosServer {}

#[async_trait::async_trait]
impl Actor for PosServer {
    async fn started(&mut self, _ctx: &mut Context<Self>) -> Result<()> {
        debug!("PosServer starting...");
        Ok(())
    }

    async fn stopped(&mut self, _ctx: &mut Context<Self>) {
        debug!("PosServer stopped");
    }
}

impl Service for PosServer {}
impl Default for PosServer {
    fn default() -> Self {
        PosServer {}
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
        // setup grpc server and services
        let addr = format!("{}:{}", msg.host, msg.port).parse().unwrap();

        info!("starting grpc service on: {}", addr);

        tokio::task::spawn(async move {
            let res = Server::builder()
                .add_service(PosDataServiceServer::new(PosGrpcService::default()))
                .serve(addr)
                .await;

            if res.is_err() {
                info!("grpc server stopped due to: {:?}", res.err().unwrap());
            } else {
                info!("grpc server stopped");
            }
        });

        Ok(())
    }
}
