use std::str;

use ahash::AHashSet;
use engines::bing::bing::Bing;
use engines::brave::brave::Brave;
use engines::duckduckgo::duckduckgo::DuckDuckGo;
use engines::engine_base::engine_base::SearchResult;
use lazy_static::lazy_static;
use rocket::form::Form;
use rocket::response::content::{RawCss, RawHtml};
use rocket::response::stream::TextStream;
use rocket::time::Instant;
use static_files::static_files::{
    render_beginning_html, render_finished_css, render_result, render_result_engine_visibility,
};
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
    static ref HTML_END: String = read_file_contents("./src/public/html/end.html").unwrap();
    static ref TAILWIND_CSS: String = read_file_contents("./src/public/css/style.css").unwrap();
}

#[get("/style.css")]
fn get_tailwindcss() -> RawCss<&'static str> {
    RawCss(&TAILWIND_CSS)
}

#[get("/")]
async fn search_get() -> RawHtml<&'static str> {
    RawHtml(include_str!("./public/html/frontpage.html"))
}

#[derive(FromForm)]
struct Body {
    query: String,
}

macro_rules! search {
    ($engine:ident,$query_ref:expr,$tx_ref:expr) => {{
        tokio::spawn(async move {
            let mut engine = $engine::new();

            engine.search($query_ref, $tx_ref).await
        })
    }};
}

#[post("/", data = "<body>")]
async fn search_post<'a>(body: Form<Body>) -> RawHtml<TextStream![String]> {
    let query = &body.query;
    let query_brave = query.to_owned().clone();
    let query_duckduckgo = query.to_owned().clone();
    let query_bing = query.to_owned().clone();

    let mut first_result_yielded = false;
    let first_result_start = Instant::now();

    let (tx, mut rx) = mpsc::channel::<SearchResult>(16);
    let tx_brave = tx.clone();
    let tx_duckduckgo = tx.clone();
    let tx_bing = tx.clone();

    let mut bing_finished_informed = false;
    let mut brave_finished_informed = false;
    let mut duckduckgo_finished_informed = false;

    let now = Instant::now();

    let brave_task = search!(Brave, &query_brave, tx_brave);
    let bing_task = search!(Bing, &query_bing, tx_bing);
    let duckduckgo_task = search!(DuckDuckGo, &query_duckduckgo, tx_duckduckgo);

    let beginning_html = render_beginning_html(&query);

    let mut results: AHashSet<String> = AHashSet::new();

    RawHtml(TextStream! {
        yield beginning_html;

        while !brave_task.is_finished() || !duckduckgo_task.is_finished() || !bing_task.is_finished() {
            while let Some(result) = rx.recv().await {
                if results.contains(&result.url) {
                    yield render_result_engine_visibility(&result.get_html_id(), &result.engine);

                    continue;
                }

                if !first_result_yielded {
                    let diff = first_result_start.elapsed().whole_milliseconds();
                    first_result_yielded = true;

                    yield format!("<strong>Time taken: {}ms</strong>", diff);
                    yield "<style>.fake { display: none; }</style>".to_string();
                }

                if !bing_finished_informed && bing_task.is_finished() {
                    bing_finished_informed = true;

                    yield render_finished_css("bing", now.elapsed().whole_milliseconds());
                }

                if !brave_finished_informed && brave_task.is_finished() {
                    brave_finished_informed = true;

                    yield render_finished_css("brave", now.elapsed().whole_milliseconds());
                }

                if !duckduckgo_finished_informed && duckduckgo_task.is_finished() {
                    duckduckgo_finished_informed = true;

                    yield render_finished_css("duckduckgo", now.elapsed().whole_milliseconds());
                }

                yield render_result(&result);
                yield render_result_engine_visibility(&result.get_html_id(), &result.engine);

                results.insert(result.url.to_string());
            }
        }

        let diff = first_result_start.elapsed().whole_milliseconds();

        if !bing_finished_informed {
            yield render_finished_css("bing", now.elapsed().whole_milliseconds());
        }

        if !brave_finished_informed {
            yield render_finished_css("brave", now.elapsed().whole_milliseconds());
        }

        if !duckduckgo_finished_informed {
            yield render_finished_css("duckduckgo", now.elapsed().whole_milliseconds());
        }

        yield format!("<strong>End taken: {}ms</strong>", diff);
        yield HTML_END.to_string();
    })
}

#[launch]
async fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![search_post, search_get])
        .mount("/", routes![get_tailwindcss])
}
