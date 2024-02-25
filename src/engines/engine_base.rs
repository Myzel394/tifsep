pub mod engine_base {
    use core::fmt;
    use std::{
        fmt::{Debug, Display},
        hash::Hash,
        ops::Sub,
        sync::Arc,
    };

    use chrono::{DateTime, TimeDelta, TimeZone, Utc};
    use futures::{lock::Mutex, Future, StreamExt};
    use lazy_static::lazy_static;
    use phf::phf_map;
    use regex::Regex;
    use reqwest::{Error, Response};
    use rustc_hash::FxHashMap;
    use tokio::sync::mpsc::Sender;
    use urlencoding::decode;

    use crate::utils::utils::{decode_html_text, hash_string};

    lazy_static! {
        static ref STRIP: Regex = Regex::new(r"[\s\n]+").unwrap();
        static ref STRIP_HTML_TAGS: Regex =
            Regex::new(r#"<(?:"[^"]*"['"]*|'[^']*'['"]*|[^'">])+>"#).unwrap();
        static ref RELATIVE_DATETIME_PARSER: Regex =
            Regex::new(r#"(?P<amount>\d+) (?P<unit>second|minute|hour|day|week|month|year)s? ago"#)
                .unwrap();
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

    #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
    pub struct SearchResultDate {
        pub date: DateTime<Utc>,
        // true if original date wasn't available and only
        // a relative time such as "2 hours ago" was provided
        pub is_relative: bool,
    }

    #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
    pub struct SearchResult {
        pub title: String,
        pub url: String,
        pub description: String,
        pub engine: SearchEngine,
        pub image_url: Option<String>,
        pub date: Option<SearchResultDate>,
    }

    impl Hash for SearchResult {
        fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
            self.url.hash(state);
        }
    }

    impl SearchResult {
        pub fn get_html_id(&self) -> String {
            // IDs must start with a letter, so we add an "h" (html ID) to the beginning
            format!("h{:X}", hash_string(&self.url),)
        }
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
            let req = request.await.unwrap();
            let url = req.url().clone();
            let mut stream = req.bytes_stream();

            let mut debug_has_fetched_once = false;
            let mut debug_content = Vec::new();
            if cfg!(debug_assertions) {
                println!("Requesting: {}", url);
            }

            while let Some(chunk) = stream.next().await {
                let buffer = chunk.unwrap();

                self.push_packet(buffer.iter());

                if cfg!(debug_assertions) {
                    debug_content.extend(buffer);
                }

                while let Some(result) = self.parse_next() {
                    if cfg!(debug_assertions) {
                        debug_has_fetched_once = true;
                    }

                    if tx.send(result).await.is_err() {
                        return Err(());
                    }
                }
            }

            while let Some(result) = self.parse_next() {
                if cfg!(debug_assertions) {
                    debug_has_fetched_once = true;
                }

                if tx.send(result).await.is_err() {
                    return Err(());
                }
            }

            if cfg!(debug_assertions) {
                if debug_has_fetched_once {
                    println!("Finished fetching: {}", url);
                } else {
                    println!("{}", "==============");
                    println!("No results for: {}", url);
                    println!("{}", String::from_utf8_lossy(&debug_content));
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

    static UNIT_VALUES_MAP: phf::Map<&'static str, u32> = phf_map! {
        "second" => 1,
        "minute" => 60,
        "hour" => 60 * 60,
        "day" => 60 * 60 * 24,
        "week" => 60 * 60 * 24 * 7,
        "month" => 60 * 60 * 30,
        "year" => 60 * 60 * 365,
    };

    impl EnginePositions {
        pub fn new() -> Self {
            EnginePositions {
                previous_block: String::new(),
                started: false,
            }
        }

        pub fn parse_date(date: &str) -> Option<DateTime<Utc>> {
            let raw_date_stripped = date.split_whitespace().collect::<Vec<&str>>().join(" ");

            if let Some(capture) = RELATIVE_DATETIME_PARSER.captures(&raw_date_stripped) {
                let now = Utc::now();
                let amount = capture.name("amount")?.as_str().parse::<i64>().ok()?;
                let unit = capture.name("unit")?.as_str();

                let multiplier = UNIT_VALUES_MAP.get(&unit)?.clone() as i64;
                let seconds_elapsed = amount * multiplier;

                let publish_date = now - TimeDelta::seconds(seconds_elapsed);

                Some(publish_date)
            } else {
                None
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
            date_format: Option<&str>,
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
                    let image = match capture.name("image") {
                        Some(image) => Some(image.as_str().to_string()),
                        None => None,
                    };

                    let mut publish_date: Option<SearchResultDate> = None;

                    if date_format.is_some() {
                        let date = capture.name("date");
                        publish_date = match date {
                            Some(date) => {
                                match DateTime::parse_from_str(date.as_str(), date_format.unwrap())
                                {
                                    Ok(parsed_date) => Some(SearchResultDate {
                                        date: parsed_date.to_utc(),
                                        is_relative: false,
                                    }),
                                    Err(_) => match EnginePositions::parse_date(&date.as_str()) {
                                        Some(parsed_date) => Some(SearchResultDate {
                                            date: parsed_date.to_utc(),
                                            is_relative: true,
                                        }),
                                        None => None,
                                    },
                                }
                            }
                            None => None,
                        };
                    }

                    let result = SearchResult {
                        title,
                        description,
                        url,
                        engine,
                        image_url: image,
                        date: publish_date,
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
