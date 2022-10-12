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
    logger: LogCollector,
}

impl ObserveService {
    pub fn new(logger: LogCollector) -> ObserveService {
        ObserveService { logger }
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
        let log_consumer = self.logger.get_consumer();
        info!("Received STDOUT Log Request");
        thread::spawn(move || {
            for i in log_consumer.into_iter() {
                match block_on(tx.send(Ok(i))) {
                    Ok(_) => {}
                    Err(e) => {
                        error!("Failed to send LogItem. Error={:?}", e);
                    }
                }
            }
        });
        info!("finished STDOUT Log Request");
        Ok(Response::new(ReceiverStream::new(rx)))
    }
}

#[derive(Debug)]
pub struct LogCollector {
    pub producer: Sender<LogItem>,
    pub consumer: Receiver<LogItem>,
}

impl LogCollector {
    pub fn new() -> LogCollector {
        let (producer, consumer) = bounded(40);
        LogCollector { producer, consumer }
    }

    pub fn get_producer(&self) -> Sender<LogItem> {
        self.producer.clone()
    }

    pub fn get_consumer(&self) -> Receiver<LogItem> {
        self.consumer.clone()
    }

    pub fn log_line(producer: Sender<LogItem>, line: &str) {
        let unix_ts = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("System Clock went backwards");

        match producer.send(LogItem {
            line: line.to_string(),
            // TODO: milliseconds type in protobuf requires 128bit type
            timestamp: unix_ts.as_secs(),
        }) {
            Ok(_) => {
                trace!("Success: Send item via producer channel to ringbuffer");
            }
            Err(e) => {
                error!("Error! {:?}", e);
            }
        }
    }

    fn consume_line(consumer: Receiver<LogItem>) -> Option<LogItem> {
        match consumer.recv() {
            Ok(val) => {
                return Some(val);
            }
            Err(e) => {
                error!("Error: {:?}", e);
                return None;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use log::Level;
    use simplelog::SimpleLogger;

    use super::*;

    fn init_logging() {
        let logger_simple = SimpleLogger::new(
            Level::Trace.to_level_filter(),
            simplelog::Config::default(),
        );

        multi_log::MultiLogger::init(vec![logger_simple], Level::Trace)
            .unwrap();
    }

    #[tokio::test]
    async fn test_ringbuffer_queue() {
        init_logging();
        let lrb = LogCollector::new();
        let prod = lrb.get_producer();

        LogCollector::log_line(prod.clone(), "hello");
        LogCollector::log_line(prod.clone(), "aurae");
        LogCollector::log_line(prod.clone(), "bye");

        let consumer = lrb.get_consumer();

        let cur_item = LogCollector::consume_line(consumer.clone());
        assert!(cur_item.is_some());
        assert_eq!(cur_item.unwrap().line, "hello");

        let cur_item = LogCollector::consume_line(consumer.clone());
        assert!(cur_item.is_some());
        assert_eq!(cur_item.unwrap().line, "aurae");

        let cur_item = LogCollector::consume_line(consumer.clone());
        assert!(cur_item.is_some());
        assert_eq!(cur_item.unwrap().line, "bye");
    }
}
