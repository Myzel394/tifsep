pub mod engine_base {
    pub struct SearchResult {
        pub title: String,
        pub url: String,
        pub description: String,
    }

    pub trait EngineBase {
        fn get_search_results(&self) -> &Vec<SearchResult>;

        fn parse_packet<'a>(&mut self, packet: impl Iterator<Item = &'a u8>);
    }
}
