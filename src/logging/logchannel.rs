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

use std::time::SystemTime;

use crossbeam::channel::{Sender, Receiver, bounded};
use log::{trace, error};
use crate::observe::LogItem;

#[derive(Debug)]
pub struct LogChannel {
    pub producer: Sender<LogItem>,
    pub consumer: Receiver<LogItem>,
    pub name: String,
}

impl LogChannel {
    pub fn new(name: &str) -> LogChannel {
        let (producer, consumer) = bounded(40);
        LogChannel { producer, consumer, name: name.to_string() }
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
            channel: "unknown".to_string(),
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
        let lrb = LogChannel::new("Test");
        let prod = lrb.get_producer();

        LogChannel::log_line(prod.clone(), "hello");
        LogChannel::log_line(prod.clone(), "aurae");
        LogChannel::log_line(prod.clone(), "bye");

        let consumer = lrb.get_consumer();

        let cur_item = LogChannel::consume_line(consumer.clone());
        assert!(cur_item.is_some());
        assert_eq!(cur_item.unwrap().line, "hello");

        let cur_item = LogChannel::consume_line(consumer.clone());
        assert!(cur_item.is_some());
        assert_eq!(cur_item.unwrap().line, "aurae");

        let cur_item = LogChannel::consume_line(consumer.clone());
        assert!(cur_item.is_some());
        assert_eq!(cur_item.unwrap().line, "bye");
    }
}
