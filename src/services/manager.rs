//! Implementation of the manager service, with three methods avaliable
//! 
//! 1, get_version_info
//! 2. describe_host
//! 3. flow_cell positions
//! 

use manager::manager_service_server::ManagerService;
use manager::{FlowCellPosition, GetVersionInfoResponse, FlowCellPositionsResponse};
// use manager::get_version_info_response::InstallationType;
use instance::get_version_info_response::MinknowVersion;
use tonic::{Request, Response, Status};
use tokio_stream::wrappers::ReceiverStream;
use tokio::sync::mpsc;

#[derive(Debug)]
pub struct Manager{
    pub(crate) positions: [FlowCellPosition; 1]
}

pub mod manager {
    tonic::include_proto!("minknow_api.manager");
}

mod instance {
    tonic::include_proto!("minknow_api.instance");
}

#[tonic::async_trait]
impl ManagerService for Manager {
    async fn get_version_info(
        &self,
        _request: Request<manager::GetVersionInfoRequest>
    ) -> Result <Response<GetVersionInfoResponse>, Status> {
        Ok(Response::new(GetVersionInfoResponse {
            minknow: Some(MinknowVersion {
                major:4, minor:0, patch:0, full:"4.0.0".to_string()
            }),
            protocols: "0.0.0.0".to_string(),
            distribution_version: "unknown".to_string(),
            distribution_status: 0,
            guppy_build_version: "banter".to_string(),
            guppy_connected_version: "4.0.0".to_string(),
            configuration: "0.0.0.0".to_string(),
            installation_type: 0
        }))
    }

    async fn describe_host (
        &self,
        _request: Request<manager::DescribeHostRequest>
    ) -> Result <Response<manager::DescribeHostResponse>, Status> {
        unimplemented!()
    }

    type flow_cell_positionsStream = ReceiverStream<Result<FlowCellPositionsResponse, Status>>;
    
    async fn flow_cell_positions (
        &self,
        _request: Request<manager::FlowCellPositionsRequest>
    ) -> Result <Response<Self::flow_cell_positionsStream>, Status> {
        let (tx, rx) = mpsc::channel(4);
        let positions = FlowCellPositionsResponse{
            positions: self.positions.clone().to_vec(),
            total_count: 1
        };

    
        tokio::spawn(async move {

            tx.send(Ok(positions.clone())).await.unwrap();
            
        });
        Ok(Response::new(ReceiverStream::new(rx)))
    }
}