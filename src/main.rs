use crate::engines::engine_base::engine_base::EngineBase;
use std::str;
use std::sync::Arc;

use engines::brave::brave::Brave;
use futures::lock::Mutex;
use futures::StreamExt;
use lazy_static::lazy_static;
use reqwest::ClientBuilder;
use rocket::response::{
    content::{RawCss, RawHtml},
    stream::TextStream,
};

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

const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/121.0.0.0 Safari/537.3";

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

#[get("/searchquery?<query>")]
async fn hello<'a>(query: &str) -> RawHtml<TextStream![String]> {
    let query_box = query.to_string();

    let completed_ref = Arc::new(Mutex::new(false));
    let completed_ref_writer = completed_ref.clone();
    let brave_ref = Arc::new(Mutex::new(Brave::new()));
    let brave_ref_writer = brave_ref.clone();

    tokio::spawn(async move {
        let client = ClientBuilder::new().user_agent(USER_AGENT).build().unwrap();
        let response = client
            .get(format!("https://search.brave.com/search?q={}", query_box))
            .send()
            .await
            .unwrap();

        let mut stream = response.bytes_stream();
        while let Some(chunk) = stream.next().await {
            let buffer = chunk.unwrap();

            let mut brave = brave_ref_writer.lock().await;
            if let Some(result) = brave.parse_packet(buffer.iter()) {
                brave.add_result(result);

                drop(brave);
                tokio::task::yield_now().await;
            }
        }

        let mut completed = completed_ref_writer.lock().await;
        *completed = true;
    });

    let mut current_index = 0;

    RawHtml(TextStream! {
        yield HTML_BEGINNING.to_string();

        loop {
            let ddg = brave_ref.lock().await;

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
        .mount("/", routes![hello])
        .mount("/", routes![search_get])
        .mount("/", routes![get_tailwindcss])
}
