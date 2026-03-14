#![allow(clippy::result_large_err)]

use core::pin::Pin;
use std::time::Duration;

use anyhow::Result;
pub use eventsource_stream::Event as MessageEvent;
pub use eventsource_stream::EventStreamError;
use eventsource_stream::Eventsource;
use futures_core::future::BoxFuture;
use futures_core::future::Future;
use futures_core::stream::BoxStream;
use futures_core::stream::Stream;
use futures_core::task::Context;
use futures_core::task::Poll;
use pin_project_lite::pin_project;
use reqwest::Error as ReqwestError;
use reqwest::Response;
use reqwest::StatusCode;
use reqwest::header::HeaderValue;
use reqwest_middleware::Error as ReqwestMiddlewareError;
use reqwest_middleware::RequestBuilder;

use super::error::EventSourceError;
use super::retry::DEFAULT_RETRY;
use super::retry::RetryPolicy;

type ResponseFuture = BoxFuture<'static, Result<Response, ReqwestMiddlewareError>>;
type EventStream = BoxStream<'static, Result<MessageEvent, EventStreamError<ReqwestError>>>;

type BoxedRetry = Box<dyn RetryPolicy + Send + Unpin + 'static>;

pin_project! {
  #[project = EventSourceProjection]

  pub struct EventSource {
      #[pin]
      next_response: Option<ResponseFuture>,
      #[pin]
      cur_stream: Option<EventStream>,
      is_closed: bool,
      retry_policy: BoxedRetry,
      last_event_id: String,
      last_retry: Option<(usize, Duration)>
  }
}

impl EventSource {
  pub fn new(builder: RequestBuilder) -> Result<Self> {
    let builder = builder
      .header(reqwest::header::ACCEPT, HeaderValue::from_static("text/event-stream"))
      .timeout(Duration::from_secs(15 * 60));
    let res_future = Box::pin(builder.send());

    Ok(Self {
      next_response: Some(res_future),
      cur_stream:    None,
      is_closed:     false,
      retry_policy:  Box::new(DEFAULT_RETRY),
      last_event_id: String::new(),
      last_retry:    None,
    })
  }

  pub fn set_retry_policy(&mut self, policy: BoxedRetry) {
    self.retry_policy = policy
  }
}

fn check_response(response: Response) -> std::result::Result<Response, EventSourceError> {
  let status_code = response.status();
  if status_code != StatusCode::OK {
    let status_text = response.status().to_string();
    let body = tokio::task::block_in_place(|| tokio::runtime::Handle::current().block_on(response.text()));

    return match body {
      Ok(body) => Err(EventSourceError::InvalidStatusCode(status_code, status_text, body)),
      Err(err) => Err(EventSourceError::InvalidStatusCode(status_code, status_text, err.to_string())),
    };
  }

  Ok(response)

  // OpenAI doesn't sent back this header
  // let headers = response.headers().clone();
  // let content_type_header = headers.get(reqwest::header::CONTENT_TYPE);
  // let Some(content_type) = content_type_header else {
  //   return Err(EventSourceError::InvalidContentType(HeaderValue::from_static(""), response));
  // };

  // let is_event_stream = content_type
  //   .clone()
  //   .to_str()
  //   .ok()
  //   .and_then(|header_value| header_value.parse::<mime::Mime>().ok())
  //   .is_some_and(|mime_type| mime_type.type_() == mime::TEXT && mime_type.subtype() == mime::EVENT_STREAM);

  // if is_event_stream { Ok(response) } else { Err(EventSourceError::InvalidContentType(content_type.clone(), response)) }
}

impl<'a> EventSourceProjection<'a> {
  fn clear_fetch(&mut self) {
    self.next_response.take();
    self.cur_stream.take();
  }

  fn handle_response(&mut self, res: Response) {
    self.last_retry.take();
    let mut stream = res.bytes_stream().eventsource();
    stream.set_last_event_id(self.last_event_id.clone());

    self.cur_stream.replace(Box::pin(stream));
  }

  fn handle_event(&mut self, event: &MessageEvent) {
    *self.last_event_id = event.id.clone();
    if let Some(duration) = event.retry {
      self.retry_policy.set_reconnection_time(duration)
    }
  }

  fn handle_error(&mut self) {
    self.clear_fetch();

    *self.is_closed = true;
  }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Event {
  Open,

  Message(MessageEvent),
}

impl From<MessageEvent> for Event {
  fn from(event: MessageEvent) -> Self {
    Event::Message(event)
  }
}

impl Stream for EventSource {
  type Item = std::result::Result<Event, EventSourceError>;

  fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
    let mut this = self.project();

    if *this.is_closed {
      return Poll::Ready(None);
    }

    if let Some(response_future) = this.next_response.as_mut().as_pin_mut() {
      match response_future.poll(cx) {
        Poll::Ready(Ok(res)) => {
          this.clear_fetch();
          match check_response(res) {
            Ok(res) => {
              this.handle_response(res);
              return Poll::Ready(Some(Ok(Event::Open)));
            }
            Err(err) => {
              *this.is_closed = true;
              return Poll::Ready(Some(Err(err)));
            }
          }
        }
        Poll::Ready(Err(err)) => {
          let err = EventSourceError::MiddlewareTransport(err);
          this.handle_error();
          return Poll::Ready(Some(Err(err)));
        }
        Poll::Pending => {
          return Poll::Pending;
        }
      }
    }

    match this.cur_stream.as_mut().as_pin_mut().unwrap().as_mut().poll_next(cx) {
      Poll::Ready(Some(Err(err))) => {
        let err = err.into();
        this.handle_error();
        Poll::Ready(Some(Err(err)))
      }
      Poll::Ready(Some(Ok(event))) => {
        this.handle_event(&event);
        Poll::Ready(Some(Ok(event.into())))
      }
      Poll::Ready(None) => {
        let err = EventSourceError::StreamEnded;
        this.handle_error();
        Poll::Ready(Some(Err(err)))
      }
      Poll::Pending => Poll::Pending,
    }
  }
}
