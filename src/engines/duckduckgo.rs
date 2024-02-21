// Search engine parser for DuckDuckGo
pub mod duckduckgo {
    use lazy_static::lazy_static;
    use regex::Regex;
    use urlencoding::decode;

    use crate::{
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
        pub previous_block: String,
        // Holds all results until consumed by iterator
        pub results: Vec<SearchResult>,
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

    impl EngineBase for DuckDuckGo {
        fn parse_next<'a>(&mut self) -> Option<SearchResult> {
            if self.results_started {
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

                        return Some(result);
                    }
                    None => {}
                }
            }

            None
        }

        fn push_packet<'a>(&mut self, packet: impl Iterator<Item = &'a u8>) {
            let bytes: Vec<u8> = packet.map(|bit| *bit).collect();
            let raw_text = String::from_utf8_lossy(&bytes);
            let text = STRIP.replace_all(&raw_text, " ");

            if self.results_started {
                self.previous_block.push_str(&text);
            } else {
                self.results_started = RESULTS_START.is_match(&text);
            }
        }
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
                results: vec![],
                completed: false,
            }
        }

        pub fn set_callback(&mut self, callback: CallbackType) {
            self.callback = callback;
        }
    }
}
