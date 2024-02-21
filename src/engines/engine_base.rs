pub mod engine_base {
    use std::{fmt::Display, sync::Arc};

    use futures::{lock::Mutex, Future, StreamExt};
    use lazy_static::lazy_static;
    use regex::Regex;
    use reqwest::{Error, Response};
    use tokio::sync::mpsc::Sender;
    use urlencoding::decode;

    use crate::utils::utils::decode_html_text;

    lazy_static! {
        static ref STRIP: Regex = Regex::new(r"[\s\n]+").unwrap();
        static ref STRIP_HTML_TAGS: Regex =
            Regex::new(r#"<(?:"[^"]*"['"]*|'[^']*'['"]*|[^'">])+>"#).unwrap();
    }

    #[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
    pub enum SearchEngine {
        Brave,
        Bing,
        DuckDuckGo,
    }

    impl Display for SearchEngine {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                SearchEngine::Brave => write!(f, "Brave"),
                SearchEngine::DuckDuckGo => write!(f, "DuckDuckGo"),
                SearchEngine::Bing => write!(f, "Bing"),
            }
        }
    }

    #[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
    pub struct SearchResult {
        pub title: String,
        pub url: String,
        pub description: String,
        pub engine: SearchEngine,
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
        ) -> Result<(), ()> {
            let mut stream = request.await.unwrap().bytes_stream();

            while let Some(chunk) = stream.next().await {
                let buffer = chunk.unwrap();

                self.push_packet(buffer.iter());

                while let Some(result) = self.parse_next() {
                    if tx.send(result).await.is_err() {
                        return Err(());
                    }
                }
            }

            while let Some(result) = self.parse_next() {
                if tx.send(result).await.is_err() {
                    return Err(());
                }
            }

            Ok(())
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

        pub fn handle_start_check_using_default_method<'a>(
            &mut self,
            results_start_regex: &Regex,
            packet: impl Iterator<Item = &'a u8>,
        ) {
            let bytes: Vec<u8> = packet.map(|bit| *bit).collect();
            let raw_text = String::from_utf8_lossy(&bytes);
            let text = STRIP.replace_all(&raw_text, " ");

            if self.started {
                self.previous_block.push_str(&text);
            } else {
                self.started = results_start_regex.is_match(&text);
            }
        }

        pub fn handle_block_using_default_method(
            &mut self,
            single_result_regex: &Regex,
            engine: SearchEngine,
        ) -> Option<SearchResult> {
            if self.started {
                if let Some(capture) = single_result_regex.captures(&self.previous_block.to_owned())
                {
                    let title = decode(capture.name("title").unwrap().as_str())
                        .unwrap()
                        .into_owned();
                    let description_raw =
                        decode_html_text(capture.name("description").unwrap().as_str()).unwrap();
                    let description = STRIP_HTML_TAGS
                        .replace_all(&description_raw, "")
                        .into_owned();
                    let url = decode(capture.name("url").unwrap().as_str())
                        .unwrap()
                        .into_owned();

                    let result = SearchResult {
                        title,
                        description,
                        url,
                        engine,
                    };

                    let end_position = capture.get(0).unwrap().end();
                    self.slice_remaining_block(&end_position);

                    return Some(result);
                }
            }

            None
        }
    }
}
