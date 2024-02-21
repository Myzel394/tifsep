pub mod engine_base {
    use std::sync::Arc;

    use futures::{lock::Mutex, Future, StreamExt};
    use lazy_static::lazy_static;
    use regex::Regex;
    use reqwest::{Error, Response};
    use tokio::sync::mpsc::Sender;

    lazy_static! {
        static ref STRIP: Regex = Regex::new(r"\s+").unwrap();
    }

    #[derive(Clone, Copy, Debug, Hash)]
    pub enum SearchEngine {
        DuckDuckGo,
    }

    #[derive(Clone, Debug, Hash)]
    pub struct SearchResult {
        pub title: String,
        pub url: String,
        pub description: String,
        pub engine: SearchEngine,
    }

    /// ResultsCollector collects results across multiple tasks
    #[derive(Clone, Debug, Hash, Default)]
    pub struct ResultsCollector {
        pub started: bool,
        pub previous_block: String,
        results: Vec<SearchResult>,
        current_index: usize,
    }

    pub trait EngineBase {
        fn parse_next<'a>(&mut self) -> Option<SearchResult>;

        fn push_packet<'a>(&mut self, packet: impl Iterator<Item = &'a u8>);

        /// Push packet to internal block and return next available search result, if available
        fn parse_packet<'a>(
            &mut self,
            packet: impl Iterator<Item = &'a u8>,
        ) -> Option<SearchResult> {
            self.push_packet(packet);

            self.parse_next()
        }

        async fn handle_request(
            &mut self,
            request: impl Future<Output = Result<Response, Error>>,
            tx: Sender<SearchResult>,
        ) {
            let mut stream = request.await.unwrap().bytes_stream();

            while let Some(chunk) = stream.next().await {
                let buffer = chunk.unwrap();

                self.push_packet(buffer.iter());

                while let Some(result) = self.parse_next() {
                    tx.send(result).await;
                }
            }

            while let Some(result) = self.parse_next() {
                tx.send(result).await;
            }
        }
    }

    impl ResultsCollector {
        pub fn new() -> Self {
            Self {
                results: Vec::new(),
                current_index: 0,
                previous_block: String::new(),
                started: false,
            }
        }

        pub fn results(&self) -> &Vec<SearchResult> {
            &self.results
        }

        pub fn add_result(&mut self, result: SearchResult) {
            self.results.push(result);
        }

        pub fn get_next_items(&self) -> &[SearchResult] {
            if self.current_index >= self.results.len() {
                return &[];
            }

            &self.results[self.current_index + 1..self.results.len()]
        }

        pub fn update_index(&mut self) {
            self.current_index = self.results.len() - 1;
        }

        pub fn has_more_results(&self) -> bool {
            if self.results.len() == 0 {
                return true;
            }

            self.current_index < self.results.len() - 1
        }
    }

    #[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
    pub struct EnginePositions {
        pub previous_block: String,
        pub started: bool,
    }

    impl EnginePositions {
        pub fn new() -> Self {
            EnginePositions {
                previous_block: String::new(),
                started: false,
            }
        }

        pub fn slice_remaining_block(&mut self, start_position: &usize) {
            let previous_block_bytes = self.previous_block.as_bytes().to_vec();
            let remaining_bytes = previous_block_bytes[*start_position..].to_vec();
            let remaining_text = String::from_utf8(remaining_bytes).unwrap();

            self.previous_block.clear();
            self.previous_block.push_str(&remaining_text);
        }
    }
}
