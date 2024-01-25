use client::client::{Client, PACKET_SIZE};

pub mod client;

fn handle_response(packet: &[u8; PACKET_SIZE], bytes: &[u8]) {
    println!("===========");
    let response = String::from_utf8_lossy(packet);
    dbg!(&packet.len());
    println!("{}", response);
}

fn main() {
    let client = Client::new("https://www.google.com/");
    client.request(&"GET", handle_response);
}
