#![allow(unused)]

use std::time::Duration;

use super::error::EventSourceError;

pub trait RetryPolicy {
  fn retry(&self, error: &EventSourceError, last_retry: Option<(usize, Duration)>) -> Option<Duration>;

  fn set_reconnection_time(&mut self, duration: Duration);
}

#[derive(Debug, Clone)]
pub struct ExponentialBackoff {
  pub start:        Duration,
  pub factor:       f64,
  pub max_duration: Option<Duration>,
  pub max_retries:  Option<usize>,
}

impl ExponentialBackoff {
  pub const fn new(start: Duration, factor: f64, max_duration: Option<Duration>, max_retries: Option<usize>) -> Self {
    Self { start, factor, max_duration, max_retries }
  }
}

impl RetryPolicy for ExponentialBackoff {
  fn retry(&self, _error: &EventSourceError, last_retry: Option<(usize, Duration)>) -> Option<Duration> {
    if let Some((retry_num, last_duration)) = last_retry {
      if self.max_retries.is_none() || retry_num < self.max_retries.unwrap() {
        let duration = last_duration.mul_f64(self.factor);
        if let Some(max_duration) = self.max_duration { Some(duration.min(max_duration)) } else { Some(duration) }
      } else {
        None
      }
    } else {
      Some(self.start)
    }
  }

  fn set_reconnection_time(&mut self, duration: Duration) {
    self.start = duration;
    if let Some(max_duration) = self.max_duration {
      self.max_duration = Some(max_duration.max(duration))
    }
  }
}

#[derive(Debug, Clone)]
pub struct Constant {
  pub delay:       Duration,
  pub max_retries: Option<usize>,
}

impl Constant {
  pub const fn new(delay: Duration, max_retries: Option<usize>) -> Self {
    Self { delay, max_retries }
  }
}

impl RetryPolicy for Constant {
  fn retry(&self, _error: &EventSourceError, last_retry: Option<(usize, Duration)>) -> Option<Duration> {
    if let Some((retry_num, _)) = last_retry {
      if self.max_retries.is_none() || retry_num < self.max_retries.unwrap() { Some(self.delay) } else { None }
    } else {
      Some(self.delay)
    }
  }

  fn set_reconnection_time(&mut self, duration: Duration) {
    self.delay = duration;
  }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Never;

impl RetryPolicy for Never {
  fn retry(&self, _error: &EventSourceError, _last_retry: Option<(usize, Duration)>) -> Option<Duration> {
    None
  }

  fn set_reconnection_time(&mut self, _duration: Duration) {}
}

pub const DEFAULT_RETRY: ExponentialBackoff =
  ExponentialBackoff::new(Duration::from_millis(300), 2., Some(Duration::from_secs(5)), None);
