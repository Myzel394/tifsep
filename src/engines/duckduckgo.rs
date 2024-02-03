// Search engine parser for DuckDuckGo
pub mod duckduckgo {
    use std::{
        cmp::min,
        collections::VecDeque,
        pin::Pin,
        str::Bytes,
        task::{Context, Poll},
    };

    use async_trait::async_trait;
    use futures::Stream;
    use lazy_static::lazy_static;
    use regex::Regex;
    use urlencoding::decode;

    use crate::{
        client::client::{Client, PACKET_SIZE},
        engines::engine_base::engine_base::{EngineBase, SearchEngine, SearchResult},
        utils::utils::decode_html_text,
    };

    lazy_static! {
        static ref RESULTS_START: Regex = Regex::new(r#"id=\"links\""#).unwrap();
        static ref SINGLE_RESULT: Regex = Regex::new(r#"<div class="result results_links.*?<a.*?href="(?P<url>.*?)".*?>(?P<title>.*?)</a>.*?class="result__snippet".*?>(?P<description>.*?)</a>.*?class="clear".*?</div>(?P<end> </div>){2}"#).unwrap();
        static ref STRIP: Regex = Regex::new(r"\s+").unwrap();
        static ref STRIP_HTML_TAGS: Regex = Regex::new(r#"<(?:"[^"]*"['"]*|'[^']*'['"]*|[^'">])+>"#).unwrap();
    }

    pub type CallbackType = Box<dyn FnMut(SearchResult) -> () + Send + Sync>;

    pub struct DuckDuckGo {
        callback: CallbackType,
        pub completed: bool,
        results_started: bool,
        previous_block: String,
        // Holds all results until consumed by iterator
        pub results: VecDeque<SearchResult>,
    }

    impl DuckDuckGo {
        fn slice_remaining_block(&mut self, start_position: &usize) {
            let previous_block_bytes = self.previous_block.as_bytes().to_vec();
            let remaining_bytes = previous_block_bytes[*start_position..].to_vec();
            let remaining_text = String::from_utf8(remaining_bytes).unwrap();

            self.previous_block.clear();
            self.previous_block.push_str(&remaining_text);
        }

        pub fn new() -> Self {
            Self {
                callback: Box::new(|_: SearchResult| {}),
                results_started: false,
                previous_block: String::new(),
                results: VecDeque::new(),
                completed: false,
            }
        }

        pub fn set_callback(&mut self, callback: CallbackType) {
            self.callback = callback;
        }
    }

    // impl Stream for DuckDuckGo {
    //     type Item = String;
    //
    //     fn poll_next(
    //         self: Pin<&mut Self>,
    //         cx: &mut Context<'_>,
    //     ) -> std::task::Poll<Option<Self::Item>> {
    //         if self.results.len() > 0 {
    //             let result = &mut self.results.pop_front().unwrap();
    //
    //             let html = format!("<br><h2>{}</h2><p>{}</p>", result.title, result.description);
    //
    //             return Poll::Ready(Some(html));
    //         }
    //
    //         if self.completed {
    //             return Poll::Ready(None);
    //         }
    //
    //         Poll::Pending
    //     }
    // }

    // impl Iterator for DuckDuckGo {
    //     type Item = SearchResult;
    //
    //     fn next(&mut self) -> Option<SearchResult> {
    //         if self.results.len() > 0 {
    //             let oldest = self.results.pop_front().unwrap();
    //
    //             Some(oldest)
    //         } else {
    //             None
    //         }
    //     }
    // }

    #[async_trait]
    impl EngineBase for DuckDuckGo {
        fn search(&mut self, query: &str) {
            dbg!("searching duckduckgo");

            let client = Client::new("https://html.duckduckgo.com/html/");

            let packets = client.request(&"POST").unwrap();

            for ii in (0..packets.len()).step_by(PACKET_SIZE) {
                let end_range = min(packets.len(), ii + PACKET_SIZE);

                let slice = &packets[ii..end_range];
                self.parse_packet(slice.iter());

                // Call callback, there is probably a better way to do this
                // while self.results.len() > 0 {
                //     let result = self.results.pop_front().unwrap();
                //
                //     (self.callback)(result);
                // }
            }

            self.completed = true;
        }

        fn parse_packet<'a>(&mut self, packet: impl Iterator<Item = &'a u8>) {
            let bytes: Vec<u8> = packet.map(|bit| *bit).collect();
            let raw_text = String::from_utf8_lossy(&bytes);
            let text = STRIP.replace_all(&raw_text, " ");

            if self.results_started {
                self.previous_block.push_str(&text);

                match SINGLE_RESULT.captures(&self.previous_block.to_owned()) {
                    Some(captures) => {
                        let title = decode(captures.name("title").unwrap().as_str())
                            .unwrap()
                            .into_owned();
                        let description_raw =
                            decode_html_text(captures.name("description").unwrap().as_str())
                                .unwrap();
                        let description = STRIP_HTML_TAGS
                            .replace_all(&description_raw, "")
                            .into_owned();
                        let url = decode(captures.name("url").unwrap().as_str())
                            .unwrap()
                            .into_owned();

                        let result = SearchResult {
                            title,
                            description,
                            url,
                            engine: SearchEngine::DuckDuckGo,
                        };

                        let end_position = captures.name("end").unwrap().end();
                        self.slice_remaining_block(&end_position);

                        // (self.callback)(result);

                        self.results.push_back(result);
                    }
                    None => {}
                }
            } else if RESULTS_START.is_match(&text) {
                self.results_started = true;
            }
        }
    }
}
