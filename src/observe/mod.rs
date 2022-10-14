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

tonic::include_proto!("observe");

use std::thread;
use std::time::SystemTime;

use crossbeam::channel::bounded;
use futures::executor::block_on;
use log::error;
use log::info;
use log::trace;

use crossbeam::channel::{unbounded, Receiver, Sender};

use crate::codes::*;
use crate::meta;
use crate::observe::observe_server::Observe;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status};

#[derive(Debug)]
pub struct ObserveService {
    consumer: Receiver<LogItem>,
}

impl ObserveService {
    pub fn new(consumer: Receiver<LogItem>) -> ObserveService {
        ObserveService { consumer }
    }
}

#[tonic::async_trait]
impl Observe for ObserveService {
    async fn status(
        &self,
        _request: Request<StatusRequest>,
    ) -> Result<Response<StatusResponse>, Status> {
        let meta = vec![meta::AuraeMeta {
            name: "UNKNOWN_NAME".to_string(),
            code: CODE_SUCCESS,
            message: "UNKNOWN_MESSAGE".to_string(),
        }];
        let response = StatusResponse { meta };
        Ok(Response::new(response))
    }

    type StdoutStream = ReceiverStream<Result<LogItem, Status>>;

    async fn stdout(
        &self,
        _request: Request<StdoutRequest>,
    ) -> Result<Response<Self::StdoutStream>, Status> {
        let (tx, rx) = mpsc::channel::<Result<LogItem, Status>>(4);
        let log_consumer = self.consumer.clone();
        thread::spawn(move || {
            for i in log_consumer.into_iter() {
                match block_on(tx.send(Ok(i))) {
                    Ok(_) => {}
                    Err(_) => {}
                }
            }
        });
        Ok(Response::new(ReceiverStream::new(rx)))
    }
}
