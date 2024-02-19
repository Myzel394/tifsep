use std::str;
use std::sync::Arc;

use engines::brave::brave::Brave;
use futures::lock::Mutex;
use lazy_static::lazy_static;
use reqwest::ClientBuilder;
use rocket::response::content::{RawCss, RawHtml};
use rocket::response::stream::TextStream;
use rocket::time::Instant;
use utils::utils::Yieldable;

use crate::helpers::helpers::run_search;
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

    let completed_ref = Arc::new(Mutex::new(false));
    let completed_ref_writer = completed_ref.clone();
    let brave_ref = Arc::new(Mutex::new(Brave::new()));
    let brave_ref_writer = brave_ref.clone();
    let mut brave_first_result_has_yielded = false;
    let brave_first_result_start = Instant::now();
    let client = Arc::new(Box::new(
        ClientBuilder::new().user_agent(USER_AGENT).build().unwrap(),
    ));
    let client_ref = client.clone();

    tokio::spawn(async move {
        let request = client_ref
            .get(format!("https://search.brave.com/search?q={}", query_box))
            .send();

        run_search(request, brave_ref_writer).await;

        let mut completed = completed_ref_writer.lock().await;
        *completed = true;
    });

    let mut current_index = 0;

    RawHtml(TextStream! {
        yield HTML_BEGINNING.to_string();

        loop {
            let brave = brave_ref.lock().await;

            let len = brave.results.len();

            if len == 0 {
                drop(brave);
                tokio::task::yield_now().await;
                continue
            }

            let completed = completed_ref.lock().await;
            if *completed && current_index == len - 1 {
                break
            }
            drop(completed);

            if !brave_first_result_has_yielded {
                let diff = brave_first_result_start.elapsed().whole_milliseconds();
                brave_first_result_has_yielded = true;

                yield format!("<strong>Time taken: {}ms</strong>", diff);
            }

            for ii in (current_index + 1)..len {
                let result = brave.results.get(ii).unwrap();

                let text = format!("<li><h1>{}</h1><p>{}</p></li>", &result.title, &result.description);

                yield text.to_string();
            }
            drop(brave);
            tokio::task::yield_now().await;

            // [1] -> 0
            // 1 -> [1]
            current_index = len - 1;
        }

        let diff = brave_first_result_start.elapsed().whole_milliseconds();
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
