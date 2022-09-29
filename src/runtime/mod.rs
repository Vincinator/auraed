/* -------------------------------------------------------------------------- *\
 *             Apache 2.0 License Copyright © 2022 The Aurae Authors          *
 *                                                                            *
 *                +--------------------------------------------+              *
 *                |   █████╗ ██╗   ██╗██████╗  █████╗ ███████╗ |              *
 *                |  ██╔══██╗██║   ██║██╔══██╗██╔══██╗██╔════╝ |              *
 *                |  ███████║██║   ██║██████╔╝███████║█████╗   |              *
 *                |  ██╔══██║██║   ██║██╔══██╗██╔══██║██╔══╝   |              *
 *                |  ██║  ██║╚██████╔╝██║  ██║██║  ██║███████╗ |              *
 *                |  ╚═╝  ╚═╝ ╚═════╝ ╚═╝  ╚═╝╚═╝  ╚═╝╚══════╝ |              *
 *                +--------------------------------------------+              *
 *                                                                            *
 *                         Distributed Systems Runtime                        *
 *                                                                            *
 * -------------------------------------------------------------------------- *
 *                                                                            *
 *   Licensed under the Apache License, Version 2.0 (the "License");          *
 *   you may not use this file except in compliance with the License.         *
 *   You may obtain a copy of the License at                                  *
 *                                                                            *
 *       http://www.apache.org/licenses/LICENSE-2.0                           *
 *                                                                            *
 *   Unless required by applicable law or agreed to in writing, software      *
 *   distributed under the License is distributed on an "AS IS" BASIS,        *
 *   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. *
 *   See the License for the specific language governing permissions and      *
 *   limitations under the License.                                           *
 *                                                                            *
\* -------------------------------------------------------------------------- */
#![allow(dead_code)]
tonic::include_proto!("runtime");
tonic::include_proto!("meta");

pub mod exec;

use crate::codes::*;
use crate::meta;
use crate::runtime::runtime_server::Runtime;
use sea_orm::DatabaseConnection;
use sea_orm::Set;
use tonic::{Request, Response, Status};

#[derive(Debug, Default, Clone)]
pub struct RuntimeService {}

#[tonic::async_trait]
impl Runtime for RuntimeService {
    async fn start_executable(
        &self,
        req: Request<Executable>,
    ) -> Result<Response<ExecutableStatus>, Status> {
        let r = req.into_inner();
        let mut meta = Vec::new();
        meta.push(meta::AuraeMeta {
            code: CODE_SUCCESS,
            message: STATUS_READY.into(),
        });
        let response =
            ExecutableStatus { meta, state: STATE_ACTIVE.into(), name: r.name };
        Ok(Response::new(response))
    }
    async fn stop_executable(
        &self,
        req: Request<Executable>,
    ) -> Result<Response<ExecutableStatus>, Status> {
        let r = req.into_inner();
        let mut meta = Vec::new();
        meta.push(meta::AuraeMeta {
            code: CODE_SUCCESS,
            message: STATUS_READY.into(),
        });
        let response =
            ExecutableStatus { meta, state: STATE_ACTIVE.into(), name: r.name };
        Ok(Response::new(response))
    }
    async fn register_executable(
        &self,
        req: Request<Executable>,
    ) -> Result<Response<ExecutableStatus>, Status> {
        let r = req.into_inner();
        let mut meta = Vec::new();
        meta.push(meta::AuraeMeta {
            code: CODE_SUCCESS,
            message: STATUS_READY.into(),
        });
        let response =
            ExecutableStatus { meta, state: STATE_ACTIVE.into(), name: r.name };
        Ok(Response::new(response))
    }
    async fn destroy_executable(
        &self,
        req: Request<Executable>,
    ) -> Result<Response<ExecutableStatus>, Status> {
        let r = req.into_inner();
        let mut meta = Vec::new();
        meta.push(meta::AuraeMeta {
            code: CODE_SUCCESS,
            message: STATUS_READY.into(),
        });
        let response =
            ExecutableStatus { meta, state: STATE_ACTIVE.into(), name: r.name };
        Ok(Response::new(response))
    }
}

use sea_orm::entity::prelude::*;

// TODO See if we can't use the prost::Message from the autogenerated gRPC code
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "executable")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub aurid: i32,
    pub name: String,
    pub exec: String,
    pub comment: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

pub async fn hydrate(
    // TODO Left off here. We need a better "db init" sequence... I am not convinced runtime::hydrate is the way
    // TODO We need to use the .proto files as our schema and hydrate a DB from that... we have DB maintenance to do..
    // TODO Migration docs: https://www.sea-ql.org/SeaORM/docs/migration/running-migration/
    // TODO Migration example: https://github.com/SeaQL/sea-orm/blob/master/examples/tonic_example/migration/src/m20220120_000001_create_post_table.rs
    _db: &DatabaseConnection,
) -> Result<(), Box<dyn std::error::Error>> {
    let _pid2 = ActiveModel {
        name: Set("auraed-child".to_owned()),
        ..Default::default() // no need to set primary key
    };

    //pid2.insert(db).await?;
    Ok(())
}
