// Search engine parser for DuckDuckGo
pub mod duckduckgo {
    // Results start at:
    //     <div id="links" class="results">
    // Example for a result:
    //     <div class="result results_links results_links_deep web-result ">
    //         <div class="links_main links_deep result__body">
    //             <h2 class="result__title">
    //                 <a
    //                     rel="nofollow" class="result__a"
    //                     href="https://www.speedtest.net/">
    //                     Speedtest by Ookla - The Global Broadband Speed Test
    //                 </a>
    //             </h2>
    //             <div class="result__extras">
    //                 <div class="result__extras__url">
    //                     <span class="result__icon">
    //                       <a rel="nofollow" href="https://www.speedtest.net/">
    //                         <img class="result__icon__img" width="16" height="16" alt=""
    //                           src="//external-content.duckduckgo.com/ip3/www.speedtest.net.ico" name="i15" />
    //                       </a>
    //                   </span>
    //                     <a class="result__url" href="https://www.speedtest.net/">
    //                         www.speedtest.net
    //                     </a>
    //                 </div>
    //             </div>
    //             <a
    //                 class="result__snippet"
    //                 href="https://www.speedtest.net/">
    //                     Use Speedtest on all your devices with our free desktop and mobile apps.
    //             </a>
    //             <div class="clear"></div>
    //         </div>
    //     </div>
    use lazy_static::lazy_static;
    use regex::Regex;

    use crate::engines::engine_base::engine_base::{EngineBase, SearchResult};

    lazy_static! {
        static ref RESULTS_START: Regex = Regex::new(r#"id=\"links\""#).unwrap();
        static ref SINGLE_RESULT: Regex = Regex::new(r#"<div class="result.*?<a.*?href="(?<url>.*?)".*?>(?<title>.*?)<\/a>.*?class="result__snippet".*?>(?<description>.*?)<\/a>.*?class="clear".*?<\/div>( <\/div>){2}"#).unwrap();
        static ref STRIP: Regex = Regex::new(r"\s+").unwrap();
    }

    pub struct DuckDuckGo {
        pub search_results: Vec<SearchResult>,
        results_started: bool,
        previous_block: String,
    }

    impl EngineBase for DuckDuckGo {
        fn get_search_results(&self) -> &Vec<SearchResult> {
            &self.search_results
        }

        fn parse_packet<'a>(&mut self, packet: impl Iterator<Item = &'a u8>) {
            let bytes: Vec<u8> = packet.map(|bit| *bit).collect();
            let raw_text = String::from_utf8_lossy(&bytes);
            let text = STRIP.replace_all(&raw_text, " ");

            if self.results_started {
                self.previous_block.push_str(&text);

                match SINGLE_RESULT.captures(&self.previous_block.to_owned()) {
                    Some(captures) => {
                        self.previous_block.clear();
                        println!("{}", &captures.name("title").unwrap().as_str());
                        println!("{}", &captures.name("description").unwrap().as_str());
                        println!("{}", &captures.name("url").unwrap().as_str());
                    }
                    None => {}
                }
            } else if RESULTS_START.is_match(&text) {
                self.results_started = true;
            }
        }
    }

    impl DuckDuckGo {
        pub fn new() -> Self {
            Self {
                search_results: Vec::new(),
                results_started: false,
                previous_block: String::new(),
            }
        }
    }
}
