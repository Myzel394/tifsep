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

use rocket::response::stream::{ReaderStream, TextStream};

use engines::{
    duckduckgo::duckduckgo::DuckDuckGo, engine_base::engine_base::EngineBase,
    engine_base::engine_base::SearchResult,
};
use tokio::sync::Mutex;

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

    TextStream! {
        let start = "<html><body>".to_string();
        yield start;

        let ddg_tv = Arc::new(
            Mutex::new(
                DuckDuckGo::new(),
            ),
        );
        let ddg_tv_clone = ddg_tv.clone();

        tokio::spawn(async move {
            ddg_tv_clone.lock().await.search(&query_box);
        });

        let mut last_position: i32 = -1;

        loop {
            let ddg = ddg_tv.lock().await;
            let len = ddg.results.len() as i32;

            if ddg.completed && last_position == len {
                break;
            }

            if last_position < (len - 1) {
                for i in max(0, last_position)..=(len - 1) {
                    match ddg.results.get(i as usize).clone() {
                        Some(result) => {
                            let html = format!("<br><h2>{}</h2><p>{}</p>", result.title, result.description);
                            yield html;
                        }
                        None => {
                            break;
                        }
                    }
                }

                last_position = len;
            }
        }

        let end = "</body></html>".to_string();
        yield end;
    }
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![index])
        .mount("/", routes![hello])
        .mount("/", routes![search_get])
}
