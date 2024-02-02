use std::cmp::min;

use client::client::{Client, PACKET_SIZE};
use engines::{duckduckgo::duckduckgo::DuckDuckGo, engine_base::engine_base::EngineBase};

pub mod client;
pub mod engines;
pub mod utils;

fn main() {
    let mut ddg = DuckDuckGo::new();
    let client = Client::new("https://html.duckduckgo.com/html/");

    let packets = client.request(&"POST").unwrap();

    for ii in (0..packets.len()).step_by(PACKET_SIZE) {
        let end_range = min(packets.len(), ii + PACKET_SIZE);

        let slice = &packets[ii..end_range];
        &ddg.parse_packet(slice.iter());
    }
}

// use std::cmp::min;
//
// use rocket::response::stream::TextStream;
// use rocket::tokio::time::{self, Duration};
//
// use client::client::{Client, PACKET_SIZE};
// use engines::{duckduckgo::duckduckgo::DuckDuckGo, engine_base::engine_base::EngineBase};
//
// pub mod client;
// pub mod engines;
// pub mod utils;
//
// #[macro_use]
// extern crate rocket;
//
// #[get("/")]
// fn index() -> &'static str {
//     "Hello, world!"
// }
//
// #[get("/infinite-hellos")]
// fn hello() -> TextStream![String] {
//     let mut ddg = DuckDuckGo::new();
//     let client = Client::new("https://html.duckduckgo.com/html/");
//
//     let packets = client.request(&"POST").unwrap();
//
//     TextStream! {
//         let mut interval = time::interval(Duration::from_secs(1));
//         interval.tick().await;
//
//         for ii in (0..packets.len()).step_by(PACKET_SIZE) {
//             let end_range = min(packets.len(), ii + PACKET_SIZE);
//
//             let slice = &packets[ii..end_range];
//             yield ddg.parse_packet(slice.iter()).to_string();
//         }
//     }
// }
//
// #[launch]
// fn rocket() -> _ {
//     rocket::build()
//         .mount("/", routes![index])
//         .mount("/", routes![hello])
// }
