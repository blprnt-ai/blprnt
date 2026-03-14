use anyhow::Result;
use reqwest_middleware::RequestBuilder;

use super::event_source_impl::EventSource;

pub trait RequestBuilderExt {
  fn eventsource(self) -> Result<EventSource>;
}

impl RequestBuilderExt for RequestBuilder {
  fn eventsource(self) -> Result<EventSource> {
    EventSource::new(self)
  }
}
