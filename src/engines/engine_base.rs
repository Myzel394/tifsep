pub mod engine_base {
    use std::sync::Arc;

    use bytes::Bytes;

    use futures::{lock::Mutex, Future, Stream, StreamExt};
    use lazy_static::lazy_static;
    use regex::Regex;
    use reqwest::{Client, Error, Response};

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

    pub trait EngineBase {
        fn add_result(&mut self, result: SearchResult);

        fn parse_next<'a>(&mut self) -> Option<SearchResult>;

        fn push_packet<'a>(&mut self, packet: impl Iterator<Item = &'a u8>);
        // fn push_packet<'a>(&mut self, packet: impl Iterator<Item = &'a u8>) {
        //     let bytes: Vec<u8> = packet.map(|bit| *bit).collect();
        //     let raw_text = String::from_utf8_lossy(&bytes);
        //     let text = STRIP.replace_all(&raw_text, " ");
        //
        //     if self.results_started {
        //         self.previous_block.push_str(&text);
        //     } else {
        //         self.results_started = RESULTS_START.is_match(&text);
        //     }
        // }

        /// Push packet to internal block and return next available search result, if available
        fn parse_packet<'a>(
            &mut self,
            packet: impl Iterator<Item = &'a u8>,
        ) -> Option<SearchResult> {
            self.push_packet(packet);

            self.parse_next()
        }

        async fn search(&mut self, query: &str);
    }

    #[derive(Clone, Debug, Hash, Default)]
    pub struct ResultsCollector {
        results: Vec<SearchResult>,
    }
}
