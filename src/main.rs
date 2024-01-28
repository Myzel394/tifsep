use std::{cmp::min, io::Read};

use client::client::{Client, PACKET_SIZE};
use engines::{duckduckgo::duckduckgo::DuckDuckGo, engine_base::engine_base::EngineBase};

pub mod client;
pub mod engines;

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
