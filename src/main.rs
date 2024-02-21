use std::sync::Arc;
use std::{str, thread};

use engines::brave::brave::Brave;
use engines::duckduckgo::duckduckgo::DuckDuckGo;
use engines::engine_base::engine_base::{ResultsCollector, SearchResult};
use futures::lock::Mutex;
use lazy_static::lazy_static;
use rocket::response::content::{RawCss, RawHtml};
use rocket::response::stream::TextStream;
use rocket::time::Instant;
use tokio::sync::mpsc;

use crate::static_files::static_files::read_file_contents;

pub mod client;
pub mod engines;
pub mod helpers;
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

    let mut first_result_yielded = false;
    let first_result_start = Instant::now();

    let (tx, mut rx) = mpsc::channel::<SearchResult>(16);

    tokio::spawn(async move {
        let mut brave = Brave::new();

        brave.search(&query_box, tx).await;
    });

    RawHtml(TextStream! {
        yield HTML_BEGINNING.to_string();

        while let Some(result) = rx.recv().await {
            if !first_result_yielded {
                let diff = first_result_start.elapsed().whole_milliseconds();
                first_result_yielded = true;

                yield format!("<strong>Time taken: {}ms</strong>", diff);
            }

            let text = format!("<li><h1>{}</h1><p>{}</p></li>", &result.title, &result.description);

            yield text.to_string();
        }

        let diff = first_result_start.elapsed().whole_milliseconds();
        yield format!("<strong>End taken: {}ms</strong>", diff);
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
