// Search engine parser for Brave Search
// This uses the clearnet, unlocalized version of the search engine.
pub mod brave {
    use lazy_static::lazy_static;
    use regex::Regex;
    use urlencoding::decode;

    use crate::{
        engines::engine_base::engine_base::{EngineBase, SearchEngine, SearchResult},
        utils::utils::decode_html_text,
    };

    lazy_static! {
        static ref RESULTS_START: Regex = Regex::new(r#"<body"#).unwrap();
        static ref SINGLE_RESULT: Regex = Regex::new(r#"<div class="snippet svelte-.+?<a href=.(?P<url>.+?)".+?<div class="title svelte-.+?">(?P<title>.+?)</div></div>.+?<div class="snippet-description.+?">(?P<description>.+?)</div></div>"#).unwrap();
        static ref STRIP: Regex = Regex::new(r"\s+").unwrap();
        static ref STRIP_HTML_TAGS: Regex = Regex::new(r#"<(?:"[^"]*"['"]*|'[^']*'['"]*|[^'">])+>"#).unwrap();
    }

    pub struct Brave {
        pub completed: bool,
        results_started: bool,
        pub previous_block: String,
        pub results: Vec<SearchResult>,
    }

    impl Brave {
        fn slice_remaining_block(&mut self, start_position: &usize) {
            let previous_block_bytes = self.previous_block.as_bytes().to_vec();
            let remaining_bytes = previous_block_bytes[*start_position..].to_vec();
            let remaining_text = String::from_utf8(remaining_bytes).unwrap();

            self.previous_block.clear();
            self.previous_block.push_str(&remaining_text);
        }

        pub fn new() -> Self {
            Self {
                results_started: false,
                previous_block: String::new(),
                results: vec![],
                completed: false,
            }
        }
    }

    impl EngineBase for Brave {
        fn add_result(&mut self, result: crate::engines::engine_base::engine_base::SearchResult) {
            self.results.push(result);
        }

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

                        let end_position = captures.get(0).unwrap().end();
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

        async fn search(&mut self, query: &str) {
            todo!()
        }
    }
}
