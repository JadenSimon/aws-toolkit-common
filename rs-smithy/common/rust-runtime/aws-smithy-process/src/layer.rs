// Applies a generic service layer on top of a process

use std::future::Future;
use tower::layer::Layer;
use tokio::io::AsyncWriteExt;

pub struct ProcessLayer {

}

pub struct ProcessTransportService {
    inner: crate::service::StartProcessResponse,
}

impl tower::Service<String> for ProcessTransportService {
    type Response = String;
    type Error = Box<dyn std::error::Error>;
    type Future = std::pin::Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        std::task::Poll::Ready(Ok(()))
    }

    // Sends a string (pretty much a byte buffer) over the wire
    // This assumes that a single request and response are consumed at a time.
    // Is is the responsibility of any further protocols to ensure that requests are correctly
    // pieced back together.
    fn call(&mut self, req: String) -> Self::Future {
        Box::pin(async move {
            self.inner.stdin.write_all(&req).await?;

        })
    }
}