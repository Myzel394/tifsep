// Search engine parser for Brave Search
// This uses the clearnet, unlocalized version of the search engine.
pub mod brave {
    use std::sync::Arc;

    use futures::lock::Mutex;
    use lazy_static::lazy_static;
    use regex::Regex;
    use tokio::sync::mpsc::Sender;
    use urlencoding::decode;

    use crate::{
        engines::engine_base::engine_base::{
            EngineBase, EnginePositions, ResultsCollector, SearchEngine, SearchResult,
        },
        helpers::helpers::build_default_client,
        utils::utils::decode_html_text,
    };

    lazy_static! {
        static ref RESULTS_START: Regex = Regex::new(r#"<body"#).unwrap();
        static ref SINGLE_RESULT: Regex = Regex::new(r#"<div class="snippet svelte-.+?<a href=.(?P<url>.+?)".+?<div class="title svelte-.+?">(?P<title>.+?)</div></div>.+?<div class="snippet-description.+?">(?P<description>.+?)</div></div>"#).unwrap();
        static ref STRIP: Regex = Regex::new(r"\s+").unwrap();
        static ref STRIP_HTML_TAGS: Regex = Regex::new(r#"<(?:"[^"]*"['"]*|'[^']*'['"]*|[^'">])+>"#).unwrap();
    }

    #[derive(Clone, Debug)]
    pub struct Brave {
        positions: EnginePositions,
    }

    impl EngineBase for Brave {
        fn parse_next<'a>(&mut self) -> Option<SearchResult> {
            if self.positions.started {
                if let Some(capture) =
                    SINGLE_RESULT.captures(&self.positions.previous_block.to_owned())
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
                        engine: SearchEngine::DuckDuckGo,
                    };

                    let end_position = capture.get(0).unwrap().end();
                    self.positions.slice_remaining_block(&end_position);

                    return Some(result);
                }
            }

            None
        }

        fn push_packet<'a>(&mut self, packet: impl Iterator<Item = &'a u8>) {
            let bytes: Vec<u8> = packet.map(|bit| *bit).collect();
            let raw_text = String::from_utf8_lossy(&bytes);
            let text = STRIP.replace_all(&raw_text, " ");

            if self.positions.started {
                self.positions.previous_block.push_str(&text);
            } else {
                self.positions.started = RESULTS_START.is_match(&text);
            }
        }
    }

    impl Brave {
        pub fn new() -> Self {
            Self {
                positions: EnginePositions::new(),
            }
        }

        pub async fn search(&mut self, query: &str, tx: Sender<SearchResult>) {
            let client = build_default_client();
            let request = client
                .get(format!("https://search.brave.com/search?q={}", query))
                .send();

            self.handle_request(request, tx).await;
        }
    }
}
