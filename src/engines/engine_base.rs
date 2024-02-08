pub mod engine_base {
    use async_trait::async_trait;

    #[derive(Clone, Copy, Debug, Hash)]
    pub enum SearchEngine {
        DuckDuckGo,
    }

    #[derive(Clone, Debug, Hash)]
    pub struct SearchResult {
        pub title: String,
        pub url: String,
        pub description: String,
        pub engine: SearchEngine,
    }

    #[async_trait]
    pub trait EngineBase {
        fn parse_packet<'a>(&mut self, packet: impl Iterator<Item = &'a u8>);
        async fn search(&mut self, query: &str);
    }
}
