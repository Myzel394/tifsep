// Search engine parser for Brave Search
// This uses the clearnet, unlocalized version of the search engine.
pub mod bing {
    use lazy_static::lazy_static;
    use regex::Regex;
    use tokio::sync::mpsc::Sender;

    use crate::{
        engines::engine_base::engine_base::{
            EngineBase, EnginePositions, SearchEngine, SearchResult,
        },
        helpers::helpers::build_default_client,
    };

    lazy_static! {
        static ref RESULTS_START: Regex = Regex::new(r#"id="b_results""#).unwrap();
        static ref SINGLE_RESULT: Regex = Regex::new(r#"<li class="b_algo".*?<h2.*?><a href="(?P<url>.+?)".*?>(?P<title>.+?)</a></h2>.*?((<div class="b_caption.*?<p.*?)|(<p class="b_lineclamp.*?))><span.*?</span>(?P<description>.*?)</p>.*?</li>"#).unwrap();
    }

    #[derive(Clone, Debug)]
    pub struct Bing {
        positions: EnginePositions,
    }

    impl EngineBase for Bing {
        fn parse_next<'a>(&mut self) -> Option<SearchResult> {
            self.positions
                .handle_block_using_default_method(&SINGLE_RESULT, SearchEngine::Bing)
        }

        fn push_packet<'a>(&mut self, packet: impl Iterator<Item = &'a u8>) {
            self.positions
                .handle_start_check_using_default_method(&RESULTS_START, packet)
        }
    }

    impl Bing {
        pub fn new() -> Self {
            Self {
                positions: EnginePositions::new(),
            }
        }

        pub async fn search(&mut self, query: &str, tx: Sender<SearchResult>) -> Result<(), ()> {
            let client = build_default_client();
            let request = client
                .get(format!("https://www.bing.com/search?q={}", query))
                .send();

            self.handle_request(request, tx).await
        }
    }
}
