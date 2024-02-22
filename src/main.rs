use std::str;

use engines::bing::bing::Bing;
use engines::brave::brave::Brave;
use engines::duckduckgo::duckduckgo::DuckDuckGo;
use engines::engine_base::engine_base::SearchResult;
use lazy_static::lazy_static;
use regex::Regex;
use rocket::response::content::{RawCss, RawHtml};
use rocket::response::stream::TextStream;
use rocket::time::Instant;
use static_files::static_files::{render_beginning_html, render_finished_css};
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
    static ref HTML_BEGINNING: String =
        read_file_contents("./src/public/html/beginning.html").unwrap();
    static ref SET_VALUE_REPLACE: Regex = Regex::new(r#"\{\% search_value \%\}"#).unwrap();
    static ref HTML_END: String = read_file_contents("./src/public/html/end.html").unwrap();
    static ref TAILWIND_CSS: String = read_file_contents("./src/public/css/style.css").unwrap();
    static ref FINISHED_CSS: String = read_file_contents("./src/public/css/finished.css").unwrap();
    static ref FINISHED_NAME_REPLACE: Regex = Regex::new(r#"(__engine__)"#).unwrap();
    static ref FINISHED_TIME_REPLACE: Regex = Regex::new(r#"{% time %}"#).unwrap();
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

#[get("/style.css")]
fn get_tailwindcss() -> RawCss<&'static str> {
    RawCss(&TAILWIND_CSS)
}

#[get("/searchquery?<query>")]
async fn hello<'a>(query: &str) -> RawHtml<TextStream![String]> {
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

    let brave_task = tokio::spawn(async move {
        let mut brave = Brave::new();

        brave.search(&query_brave, tx_brave).await;
    });

    let duckduckgo_task = tokio::spawn(async move {
        let mut duckduckgo = DuckDuckGo::new();

        duckduckgo.search(&query_duckduckgo, tx_duckduckgo).await;
    });

    let bing_task = tokio::spawn(async move {
        let mut bing = Bing::new();

        bing.search(&query_bing, tx_bing).await;
    });

    let beginning_html = render_beginning_html(&query);

    RawHtml(TextStream! {
        yield beginning_html;

        while !brave_task.is_finished() || !duckduckgo_task.is_finished() || !bing_task.is_finished() {
            while let Some(result) = rx.recv().await {
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

                let text = format!("<li><h1>{}</h1><p>{}</p><i>{}</i></li>", &result.title, &result.description, &result.engine.to_string());

                yield text.to_string();
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
        .mount("/", routes![hello])
        .mount("/", routes![search_get])
        .mount("/", routes![get_tailwindcss])
}
