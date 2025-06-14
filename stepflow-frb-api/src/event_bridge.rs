use futures_core::Stream;
use std::pin::Pin;
use std::task::{Context, Poll};
use stepflow_dto::dto::event_envelope::EventEnvelope;
use tokio_stream::wrappers::BroadcastStream;

pub struct FrbEventStream {
    inner: BroadcastStream<EventEnvelope>,
}

impl FrbEventStream {
    pub fn new() -> Self {
        let rx = crate::init::get_event_bus().subscribe();
        let inner = BroadcastStream::new(rx);
        Self { inner }
    }
}

impl Stream for FrbEventStream {
    type Item = EventEnvelope;

    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        match Pin::new(&mut self.inner).poll_next(cx) {
            Poll::Ready(Some(Ok(evt))) => Poll::Ready(Some(evt)),
            Poll::Ready(Some(Err(_))) => Poll::Pending,
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}