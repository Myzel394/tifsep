// Search engine parser for DuckDuckGo
pub mod duckduckgo {
    use std::{
        io::{Read, Write},
        net::TcpStream,
        sync::Arc,
    };

    use lazy_static::lazy_static;
    use regex::Regex;
    use rustls::RootCertStore;
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
        fn add_result(&mut self, result: SearchResult) {
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

        // Searches DuckDuckGo for the given query
        // Uses rustls as reqwest does not support accessing the raw packets
        async fn search(&mut self, query: &str) {
            let root_store =
                RootCertStore::from_iter(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
            let mut config = rustls::ClientConfig::builder()
                .with_root_certificates(root_store)
                .with_no_client_auth();

            // Allow using SSLKEYLOGFILE.
            config.key_log = Arc::new(rustls::KeyLogFile::new());

            let now = std::time::Instant::now();
            let server_name = "html.duckduckgo.com".try_into().unwrap();
            let mut conn = rustls::ClientConnection::new(Arc::new(config), server_name).unwrap();
            let mut sock = TcpStream::connect("html.duckduckgo.com:443").unwrap();
            let mut tls = rustls::Stream::new(&mut conn, &mut sock);
            tls.write_all(
                concat!(
                    "POST /html/ HTTP/1.1\r\n",
                    "Host: html.duckduckgo.com\r\n",
                    "Connection: cloSe\r\n",
                    "Accept-Encoding: identity\r\n",
                    "Content-Length: 6\r\n",
                    // form data
                    "Content-Type: application/x-www-form-urlencoded\r\n",
                    "\r\n",
                    "q=test",
                )
                .as_bytes(),
            )
            .unwrap();
            let mut plaintext = Vec::new();
            dbg!(now.elapsed());

            loop {
                let mut buf = [0; 65535];
                tls.conn.complete_io(tls.sock);
                let n = tls.conn.reader().read(&mut buf);

                if n.is_ok() {
                    dbg!(&n);
                    let n = n.unwrap();
                    if n == 0 {
                        break;
                    }
                    println!("{}", "=================");
                    dbg!(now.elapsed());
                    // println!("{}", String::from_utf8_lossy(&buf));
                    plaintext.extend_from_slice(&buf);
                }
            }

            // let root_store = RootCertStore {
            //     roots: webpki_roots::TLS_SERVER_ROOTS.into(),
            // };
            //
            // let mut config = rustls::ClientConfig::builder()
            //     .with_root_certificates(root_store)
            //     .with_no_client_auth();
            //
            // // Allow using SSLKEYLOGFILE.
            // config.key_log = Arc::new(rustls::KeyLogFile::new());
            //
            // let server_name = "html.duckduckgo.com".try_into().unwrap();
            // let mut conn = rustls::ClientConnection::new(Arc::new(config), server_name).unwrap();
            //
            // let mut sock = TcpStream::connect("html.duckduckgo.com:443").unwrap();
            // let mut tls = rustls::Stream::new(&mut conn, &mut sock);
            // tls.write_all(
            //     concat!(
            //         "POST /html/ HTTP/1.1\r\n",
            //         "Host: html.duckduckgo.com\r\n",
            //         "Connection: close\r\n",
            //         "Accept-Encoding: identity\r\n",
            //         "Content-Length: 6\r\n",
            //         // form data
            //         "Content-Type: application/x-www-form-urlencoded\r\n",
            //         "\r\n",
            //         "q=test",
            //     )
            //     .as_bytes(),
            // )
            // .unwrap();
            // let ciphersuite = tls.conn.negotiated_cipher_suite().unwrap();
            // writeln!(
            //     &mut std::io::stderr(),
            //     "Current ciphersuite: {:?}",
            //     ciphersuite.suite()
            // )
            // .unwrap();
            //
            // // Iterate over the stream to read the response.
            // loop {
            //     let mut buf = [0u8; 1024];
            //     let n = tls.read(&mut buf).unwrap();
            //     if n == 0 {
            //         break;
            //     }
            //
            //     if let Some(result) = self.parse_packet(buf.iter()) {
            //         self.add_result(result);
            //
            //         // Wait one second
            //         std::thread::sleep(std::time::Duration::from_millis(100));
            //     }
            // }
            //
            // while let Some(result) = self.parse_next() {
            //     self.add_result(result);
            // }
            //
            // dbg!("done with searching");

            // let client = reqwest::Client::new();
            //
            // let now = std::time::Instant::now();
            //
            // let mut stream = client
            //     .post("https://html.duckduckgo.com/html/")
            //     .header("Content-Type", "application/x-www-form-urlencoded")
            //     .body(format!("q={}", query))
            //     .send()
            //     .await
            //     .unwrap()
            //     .bytes_stream();
            //
            // let diff = now.elapsed();
            // dbg!(diff);
            //
            // while let Some(item) = stream.next().await {
            //     let packet = item.unwrap();
            //
            //     if let Some(result) = self.parse_packet(packet.iter()) {
            //         self.add_result(result);
            //     }
            // }
            //
            // while let Some(result) = self.parse_next() {
            //     self.add_result(result);
            // }
            //
            // let second_diff = now.elapsed();
            // dbg!(second_diff);
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
