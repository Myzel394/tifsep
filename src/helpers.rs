// Helpers for specific project-related tasks
// This module differs from utils in the way that everything here
// is specifically related the project
pub mod helpers {
    use std::sync::Arc;

    use futures::{lock::Mutex, Future, StreamExt};
    use reqwest::{Error, Response};

    use crate::engines::engine_base::engine_base::EngineBase;

    pub async fn run_search(
        request: impl Future<Output = Result<Response, Error>>,
        engine_ref: Arc<Mutex<impl EngineBase>>,
    ) {
        let response = request.await.unwrap();

        let mut stream = response.bytes_stream();
        while let Some(chunk) = stream.next().await {
            let buffer = chunk.unwrap();

            let mut engine = engine_ref.lock().await;

            if let Some(result) = engine.parse_packet(buffer.iter()) {
                engine.add_result(result);

                drop(engine);
                tokio::task::yield_now().await;
            }
        }
    }
}
