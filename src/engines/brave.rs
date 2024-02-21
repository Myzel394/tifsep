// Search engine parser for Brave Search
// This uses the clearnet, unlocalized version of the search engine.
pub mod brave {
    use lazy_static::lazy_static;
    use regex::Regex;
    use tokio::sync::mpsc::Sender;

    use crate::{
        engines::engine_base::engine_base::{EngineBase, EnginePositions, SearchResult},
        helpers::helpers::build_default_client,
    };

    lazy_static! {
        static ref RESULTS_START: Regex = Regex::new(r#"<body"#).unwrap();
        static ref SINGLE_RESULT: Regex = Regex::new(r#"<div class="snippet svelte-.+?<a href=.(?P<url>.+?)".+?<div class="title svelte-.+?">(?P<title>.+?)</div></div>.+?<div class="snippet-description.+?">(?P<description>.+?)</div></div>"#).unwrap();
        static ref STRIP: Regex = Regex::new(r"\s+").unwrap();
    }

    #[derive(Clone, Debug)]
    pub struct Brave {
        positions: EnginePositions,
    }

    impl EngineBase for Brave {
        fn parse_next<'a>(&mut self) -> Option<SearchResult> {
            self.positions
                .handle_block_using_default_method(&SINGLE_RESULT)
        }

        fn push_packet<'a>(&mut self, packet: impl Iterator<Item = &'a u8>) {
            self.positions
                .handle_start_check_using_default_method(&RESULTS_START, packet)
        }
    }

    impl Brave {
        pub fn new() -> Self {
            Self {
                positions: EnginePositions::new(),
            }
        }

        pub async fn search(&mut self, query: &str, tx: Sender<SearchResult>) -> Result<(), ()> {
            let client = build_default_client();
            let request = client
                .get(format!("https://search.brave.com/search?q={}", query))
                .send();

            self.handle_request(request, tx).await
        }
    }
}
