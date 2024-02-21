// Helpers for specific project-related tasks
// This module differs from utils in the way that everything here
// is specifically related the project
pub mod helpers {
    use std::sync::Arc;

    use bytes::Bytes;
    use futures::{lock::Mutex, Future, Stream, StreamExt};
    use reqwest::{Client, ClientBuilder, Error, Response};

    use crate::engines::engine_base::engine_base::{EngineBase, ResultsCollector};

    const DEFAULT_USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/121.0.0.0 Safari/537.3";

    pub fn build_default_client() -> Client {
        ClientBuilder::new()
            .user_agent(DEFAULT_USER_AGENT)
            .build()
            .unwrap()
    }
}
