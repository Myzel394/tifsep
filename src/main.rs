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
#![feature(async_closure)]

use std::sync::Arc;

use futures::lock::Mutex;
use lazy_static::lazy_static;
use rocket::response::{content::RawHtml, stream::TextStream};

use engines::duckduckgo::duckduckgo::DuckDuckGo;

use crate::static_files::static_files::read_file_contents;

pub mod client;
pub mod engines;
pub mod static_files;
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
fn get_tailwindcss() -> &'static str {
    &TAILWIND_CSS
}

#[get("/searchquery?<query>")]
async fn hello<'a>(query: &str) -> RawHtml<TextStream![String]> {
    let query_box = Box::new(query.to_string());

    let ddg_ref = Arc::new(Mutex::new(DuckDuckGo::new()));
    let ddg_ref_writer = ddg_ref.clone();

    tokio::spawn(async move {
        let mut ddg = ddg_ref_writer.lock().await;

        ddg.search(&query_box).await;
    });

    let mut current_index = 0;

    RawHtml(TextStream! {
        yield HTML_BEGINNING.to_string();

        loop {
            let ddg = ddg_ref.lock().await;

            let len = ddg.results.len();

            if len == 0 {
                continue
            }

            if ddg.completed && current_index == len - 1 {
                break
            }

            for ii in (current_index + 1)..len {
                let result = ddg.results.get(ii).unwrap();

                let text = format!("<li><h1>{}</h1><p>{}</p></li>", &result.title, &result.description);

                yield text.to_string();
            }

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
}
