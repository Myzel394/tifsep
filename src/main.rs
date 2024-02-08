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

use std::{
    cmp::max,
    sync::{Arc, RwLock},
};

use futures::lock::Mutex;
use rocket::response::stream::TextStream;

use engines::duckduckgo::duckduckgo::DuckDuckGo;

pub mod client;
pub mod engines;
pub mod utils;

#[macro_use]
extern crate rocket;

struct SearchParams<'r> {
    query: &'r str,
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

#[get("/searchquery?<query>")]
async fn hello<'a>(query: &str) -> TextStream![String] {
    let query_box = Box::new(query.to_string());

    let ddg_ref = Arc::new(Mutex::new(DuckDuckGo::new()));
    let ddg_writer_ref = ddg_ref.clone();

    tokio::spawn(async move {
        let mut ddg = ddg_writer_ref.lock().await;
        ddg.search(&query_box).await;
    });

    let mut current_index = 0;

    TextStream! {
        let start = "<DOCTYPE!html><html><body>".to_string();
        yield start;

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
                let result = ddg.results.get(ii);

                dbg!(&result);
            }

            // [1] -> 0
            // 1 -> [1]
            current_index = len - 1;
        }

        let end = "</body></html>".to_string();

        yield end
    }
}

#[launch]
async fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![index])
        .mount("/", routes![hello])
        .mount("/", routes![search_get])
}
