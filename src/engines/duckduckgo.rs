// Search engine parser for DuckDuckGo Search
pub mod duckduckgo {
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
        static ref RESULTS_START: Regex = Regex::new(r#"id=\"links\""#).unwrap();
        static ref SINGLE_RESULT: Regex = Regex::new(r#"<div class="result results_links.*?<a.*?href="(?P<url>.*?)".*?>(?P<title>.*?)</a>.*?class="result__snippet".*?>(?P<description>.*?)</a>.*?class="clear".*?</div>(?P<end> </div>){2}"#).unwrap();
    }

    const URL: &str = "https://html.duckduckgo.com/html";

    #[derive(Clone, Debug)]
    pub struct DuckDuckGo {
        positions: EnginePositions,
    }

    impl EngineBase for DuckDuckGo {
        fn parse_next<'a>(&mut self) -> Option<SearchResult> {
            self.positions.handle_block_using_default_method(
                &SINGLE_RESULT,
                SearchEngine::DuckDuckGo,
                None,
            )
        }

        fn push_packet<'a>(&mut self, packet: impl Iterator<Item = &'a u8>) {
            self.positions
                .handle_start_check_using_default_method(&RESULTS_START, packet)
        }
    }

    impl DuckDuckGo {
        pub fn new() -> Self {
            Self {
                positions: EnginePositions::new(),
            }
        }

        pub async fn search(&mut self, query: &str, tx: Sender<SearchResult>) -> Result<(), ()> {
            let client = build_default_client();
            let params = [("q", query)];
            let request = client.post(URL).form(&params).send();

            self.handle_request(request, tx).await
        }
    }
}
