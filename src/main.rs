// use engines::{
//     duckduckgo::duckduckgo::DuckDuckGo,
//     engine_base::engine_base::{EngineBase, SearchResult},
// };
//
// pub mod client;
// pub mod engines;
// pub mod utils;
//
// #[tokio::main]
// async fn main() {
//     let callback = Box::new(|result: SearchResult| {
//         dbg!(&result);
//     });
//     let mut ddg = DuckDuckGo::new(callback);
//     ddg.search(&"test").await;
//
//     println!("done");
// }

// Found no other way to make this work
use crate::engines::engine_base::engine_base::EngineBase;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::str;
use std::sync::Arc;
use std::time::Instant;

use engines::brave::brave::Brave;
use futures::lock::Mutex;
use lazy_static::lazy_static;
use rocket::response::{
    content::{RawCss, RawHtml},
    stream::TextStream,
};
use rustls::RootCertStore;

use crate::static_files::static_files::read_file_contents;

pub mod client;
pub mod engines;
pub mod static_files;
pub mod tsclient;
pub mod utils;

#[macro_use]
extern crate rocket;

lazy_static! {
    static ref HTML_BEGINNING: String = read_file_contents("./src/html/beginning.html").unwrap();
    static ref HTML_END: String = read_file_contents("./src/html/end.html").unwrap();
    static ref TAILWIND_CSS: String = read_file_contents("./tailwindcss/output.css").unwrap();
}

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[get("/search")]
fn search_get() -> &'static str {
    "<html>
        <body>
            <form method='get' action='/searchquery'>
                <input name='query'>
                <button type='submit'>Search</button>
            </form>
        </body>
    </html>"
}

#[get("/tailwind.css")]
fn get_tailwindcss() -> RawCss<&'static str> {
    RawCss(&TAILWIND_CSS)
}

#[get("/slow")]
async fn slow() -> &'static str {
    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

    "Slow"
}

#[get("/slowresponse")]
async fn slowresponse() -> TextStream![String] {
    TextStream! {
        yield "First".to_owned();

        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

        yield "second".to_owned();

        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

        yield "third".to_owned();
    }
}

#[get("/searchquery?<query>")]
async fn hello<'a>(query: &str) -> RawHtml<TextStream![String]> {
    let query_box = Box::new(query.to_string());
    let now = Arc::new(Box::new(Instant::now()));

    let completed_ref = Arc::new(Mutex::new(false));
    let completed_ref_writer = completed_ref.clone();
    let ddg_ref = Arc::new(Mutex::new(Brave::new()));
    let ddg_ref_writer = ddg_ref.clone();

    let now_ref = now.clone();

    tokio::spawn(async move {
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
        // let now = Instant::now();
        // let server_name = "html.duckduckgo.com".try_into().unwrap();
        // let mut conn = rustls::ClientConnection::new(Arc::new(config), server_name).unwrap();
        //
        // let mut sock = TcpStream::connect("html.duckduckgo.com:443".to_socket_addrs()).unwrap();
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

        // dbg!("Connected to DuckDuckGo");
        // dbg!(now.elapsed());
        //
        // // Iterate over the stream to read the response in real time
        //
        // loop {
        //     if conn.wants_read() {
        //         conn.read_tls(&mut sock).unwrap();
        //         conn.process_new_packets().unwrap();
        //
        //         let mut plaintext = Vec::new();
        //         conn.reader().read_to_end(&mut plaintext).unwrap();
        //     }
        //
        //     if conn.wants_write() {
        //         conn.write_tls(&mut sock).unwrap();
        //     }
        //     sock.wa
        // }

        // loop {
        //     dbg!(now.elapsed());
        //     let mut buf = [0u8; 1024];
        //     let n = tls.read(&mut buf).unwrap();
        //     if n == 0 {
        //         break;
        //     }
        //
        //     dbg!(now.elapsed());
        //
        //     let mut ddg = ddg_ref_writer.lock().await;
        //     if let Some(result) = ddg.parse_packet(buf.iter()) {
        //         ddg.add_result(result);
        //     }
        //
        //     // Release
        //     drop(ddg);
        //     tokio::task::yield_now().await;
        // }

        // dbg!("done with content");
        // dbg!(now.elapsed());
        //
        // let mut ddg = ddg_ref_writer.lock().await;
        // while let Some(result) = ddg.parse_next() {
        //     ddg.add_result(result);
        // }

        let root_store = RootCertStore::from_iter(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
        let mut config = rustls::ClientConfig::builder()
            .with_root_certificates(root_store)
            .with_no_client_auth();

        // Allow using SSLKEYLOGFILE.
        config.key_log = Arc::new(rustls::KeyLogFile::new());

        let server_name = "search.brave.com".try_into().unwrap();
        let mut conn = rustls::ClientConnection::new(Arc::new(config), server_name).unwrap();
        let mut sock = TcpStream::connect("search.brave.com:443").unwrap();
        let mut tls = rustls::Stream::new(&mut conn, &mut sock);
        tls.write_all(
            concat!(
                "GET /search?q=test&show_local=0&source=unlocalise HTTP/1.1\r\n",
                "Host: search.brave.com\r\n",
                "Connection: close\r\n",
                "Accept-Encoding: identity\r\n",
                "User-Agent: Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/121.0.0.0 Safari/537.3\r\n",
                "\r\n",
            )
            .as_bytes(),
        )
        .unwrap();

        loop {
            let mut buf = [0; 65535];
            tls.conn.complete_io(tls.sock);
            let n = tls.conn.reader().read(&mut buf);

            // dbg!(&n);

            if n.is_ok() {
                let n = n.unwrap();
                if n == 0 {
                    break;
                }
                // println!("{}", String::from_utf8_lossy(&buf));
                let mut brave = ddg_ref_writer.lock().await;

                if let Some(result) = brave.parse_packet(buf.iter()) {
                    println!("Brave: {}", now_ref.elapsed().as_millis());
                    brave.add_result(result);

                    drop(brave);
                    tokio::task::yield_now().await;
                }
            }
        }

        let mut completed = completed_ref_writer.lock().await;
        *completed = true;
    });

    let mut current_index = 0;

    RawHtml(TextStream! {
        yield HTML_BEGINNING.to_string();

        loop {
            let ddg = ddg_ref.lock().await;

            let len = ddg.results.len();

            if len == 0 {
            drop(ddg);
            tokio::task::yield_now().await;
                continue
            }

            let completed = completed_ref.lock().await;
            if *completed && current_index == len - 1 {
                break
            }
            drop(completed);

            for ii in (current_index + 1)..len {
                let result = ddg.results.get(ii).unwrap();

                println!("Yield: {}", now.elapsed().as_millis());
                let text = format!("<li><h1>{}</h1><p>{}</p></li>", &result.title, &result.description);

                yield text.to_string();
            }
            drop(ddg);
            tokio::task::yield_now().await;

            // [1] -> 0
            // 1 -> [1]
            current_index = len - 1;
        }

        yield HTML_END.to_string();
    })
}

#[launch]
async fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![index])
        .mount("/", routes![hello])
        .mount("/", routes![search_get])
        .mount("/", routes![get_tailwindcss])
        .mount("/", routes![slow])
        .mount("/", routes![slowresponse])
}
