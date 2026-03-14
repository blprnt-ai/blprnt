use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use std::time::Instant;

use reqwest::Response;
use tokio::sync::Mutex;

/// A rate limit window tracking remaining quota and reset time for a specific header pair.
///
/// Only `remaining_key` is used for identity/lookup. The `remaining` and `reset_at`
/// fields are mutable state that gets updated from response headers.
#[derive(Clone, Debug)]
pub struct Window {
  remaining_key: String,
  reset_key:     String,
  remaining:     Option<u64>,
  reset_at:      Option<Instant>,
}

impl Window {
  pub fn new(remaining_key: String, reset_key: String) -> Self {
    Self { remaining_key, reset_key, remaining: None, reset_at: None }
  }

  /// Returns true if this window indicates we should wait (remaining == 0 with future reset).
  fn needs_wait(&self, now: Instant) -> Option<Duration> {
    match (self.remaining, self.reset_at) {
      (Some(0), Some(reset_at)) if reset_at > now => Some(reset_at - now),
      _ => None,
    }
  }

  /// Returns true if this window's reset time has passed and should be cleared.
  fn is_stale(&self, now: Instant) -> bool {
    matches!(self.reset_at, Some(reset_at) if reset_at <= now)
  }
}

#[derive(Clone, Debug)]
struct State {
  initialized: bool,
  retry_until: Option<Instant>,
  /// Windows keyed by `remaining_key` for stable identity and in-place updates.
  windows:     HashMap<String, Window>,
}

impl State {
  pub fn new(windows: impl IntoIterator<Item = Window>) -> Self {
    let windows = windows.into_iter().map(|w| (w.remaining_key.clone(), w)).collect();
    Self { initialized: false, retry_until: None, windows }
  }
}

pub type ParseHeaderFn = fn(value: String) -> Option<time::Duration>;

/// Rate limiter that tracks provider rate limit headers and blocks requests when limits are exhausted.
///
/// Each WindowLicker instance should be scoped to a single API key/credential set.
/// Multiple concurrent requests sharing the same credentials should share a WindowLicker.
#[derive(Clone, Debug)]
pub struct WindowLicker {
  state:           Arc<Mutex<State>>,
  parse_header_fn: ParseHeaderFn,
}

impl WindowLicker {
  pub fn new(windows: impl IntoIterator<Item = Window>, parse_header_fn: ParseHeaderFn) -> Self {
    Self { state: Arc::new(Mutex::new(State::new(windows))), parse_header_fn }
  }

  /// Blocks until rate limits allow a request to proceed.
  ///
  /// - Returns immediately if not yet initialized (no headers seen yet).
  /// - Respects `retry-after` as a hard gate.
  /// - Uses minimum wait across windows (least restrictive) for advisory limits.
  pub async fn preflight(&self) {
    loop {
      {
        let state = self.state.lock().await;
        if !state.initialized {
          // First request: allow through to discover limits
          break;
        }
      }

      let wait_for = self.get_wait_for().await;

      if let Some(duration) = wait_for {
        tokio::time::sleep(duration).await;
        continue;
      }
      break;
    }
  }

  /// Updates rate limit state from response headers.
  ///
  /// Parses `retry-after` and provider-specific rate limit headers,
  /// updating window state in place.
  pub async fn update_from_response(&self, resp: &Response) {
    let mut state = self.state.lock().await;
    let now = Instant::now();

    // Parse retry-after (hard backoff from 429 responses)
    // Check for 429 before setting retry-after
    if let Some(value) = self.header_str(resp, "retry-after")
      && resp.status().as_u16() == 429
      && let Ok(seconds) = value.trim().parse::<f64>()
    {
      let until = Instant::now() + Duration::from_secs_f64(seconds.max(0.0));
      state.retry_until = Some(until);
    }

    // Update each window in place from response headers
    for window in state.windows.values_mut() {
      // Clear stale reset times
      if window.is_stale(now) {
        window.remaining = None;
        window.reset_at = None;
      }

      // Parse remaining count
      if let Some(remaining) = self.header_str(resp, &window.remaining_key).and_then(|s| self.parse_remaining(s)) {
        window.remaining = Some(remaining);
      }

      // Parse reset time
      if let Some(reset_at) = self.header_str(resp, &window.reset_key).and_then(|s| self.parse_reset_instant(s)) {
        window.reset_at = Some(reset_at);
      }
    }

    state.initialized = true;
  }

  /// Computes how long to wait before the next request, if any.
  ///
  /// Decision rules:
  /// 1. `retry_until` (from retry-after header) is a hard gate - must wait.
  /// 2. Windows with `remaining == 0` and future `reset_at` are advisory.
  /// 3. Uses MINIMUM wait across advisory windows (least restrictive).
  /// 4. Clears stale windows (reset time has passed).
  async fn get_wait_for(&self) -> Option<Duration> {
    let mut state = self.state.lock().await;
    let now = Instant::now();

    // Rule 1: retry-after is a hard gate
    if let Some(until) = state.retry_until {
      if until > now {
        return Some(until - now);
      } else {
        // Stale retry_until, clear it
        state.retry_until = None;
      }
    }

    // Collect wait durations from windows, clearing stale ones
    let wait_durations: Vec<Duration> = state
      .windows
      .values_mut()
      .filter_map(|window| {
        // Clear stale windows
        if window.is_stale(now) {
          window.remaining = None;
          window.reset_at = None;
          return None;
        }
        window.needs_wait(now)
      })
      .collect();

    // Rule 3: Use MINIMUM wait (least restrictive)
    // This allows requests that might fit within some limit
    wait_durations.into_iter().min()
  }

  fn header_str(&self, resp: &Response, name: &str) -> Option<String> {
    resp.headers().get(name).and_then(|value| value.to_str().ok()).map(|s| s.to_string())
  }

  fn parse_remaining(&self, string: String) -> Option<u64> {
    let clean = string.trim().replace(',', "");
    clean.parse().ok()
  }

  fn parse_reset_instant(&self, string: String) -> Option<Instant> {
    let duration = (self.parse_header_fn)(string)?;
    let nanos = duration.whole_nanoseconds();
    if nanos <= 0 {
      return None;
    }
    Some(Instant::now() + Duration::from_nanos(nanos as u64))
  }
}

#[cfg(test)]
mod tests {
  use std::time::Duration;

  use super::*;

  fn dummy_parse(_: String) -> Option<time::Duration> {
    Some(time::Duration::seconds(5))
  }

  #[test]
  fn window_needs_wait_when_remaining_zero_with_future_reset() {
    let now = Instant::now();
    let future = now + Duration::from_secs(10);

    let window = Window {
      remaining_key: "r".into(),
      reset_key:     "t".into(),
      remaining:     Some(0),
      reset_at:      Some(future),
    };

    let wait = window.needs_wait(now);
    assert!(wait.is_some());
    assert!(wait.unwrap() > Duration::from_secs(9));
  }

  #[test]
  fn window_no_wait_when_remaining_nonzero() {
    let now = Instant::now();
    let future = now + Duration::from_secs(10);

    let window = Window {
      remaining_key: "r".into(),
      reset_key:     "t".into(),
      remaining:     Some(5),
      reset_at:      Some(future),
    };

    assert!(window.needs_wait(now).is_none());
  }

  #[test]
  fn window_no_wait_when_remaining_none() {
    let now = Instant::now();
    let window =
      Window { remaining_key: "r".into(), reset_key: "t".into(), remaining: None, reset_at: None };

    assert!(window.needs_wait(now).is_none());
  }

  #[test]
  fn window_is_stale_when_reset_passed() {
    let now = Instant::now();
    let past = now - Duration::from_secs(10);

    let window = Window {
      remaining_key: "r".into(),
      reset_key:     "t".into(),
      remaining:     Some(0),
      reset_at:      Some(past),
    };

    assert!(window.is_stale(now));
  }

  #[test]
  fn window_not_stale_when_reset_in_future() {
    let now = Instant::now();
    let future = now + Duration::from_secs(10);

    let window = Window {
      remaining_key: "r".into(),
      reset_key:     "t".into(),
      remaining:     Some(0),
      reset_at:      Some(future),
    };

    assert!(!window.is_stale(now));
  }

  #[tokio::test]
  async fn preflight_allows_first_request_without_blocking() {
    let licker = WindowLicker::new(vec![Window::new("r".into(), "t".into())], dummy_parse);

    let start = Instant::now();
    licker.preflight().await;
    let elapsed = start.elapsed();

    // Should return immediately since not initialized
    assert!(elapsed < Duration::from_millis(100));
  }

  #[tokio::test]
  async fn get_wait_for_returns_none_when_no_limits_exhausted() {
    let licker = WindowLicker::new(
      vec![Window {
        remaining_key: "r".into(),
        reset_key:     "t".into(),
        remaining:     Some(10),
        reset_at:      None,
      }],
      dummy_parse,
    );

    // Initialize state manually
    {
      let mut state = licker.state.lock().await;
      state.initialized = true;
    }

    let wait = licker.get_wait_for().await;
    assert!(wait.is_none());
  }

  #[tokio::test]
  async fn get_wait_for_uses_minimum_across_windows() {
    let now = Instant::now();

    let licker = WindowLicker::new(
      vec![
        Window {
          remaining_key: "r1".into(),
          reset_key:     "t1".into(),
          remaining:     Some(0),
          reset_at:      Some(now + Duration::from_secs(10)),
        },
        Window {
          remaining_key: "r2".into(),
          reset_key:     "t2".into(),
          remaining:     Some(0),
          reset_at:      Some(now + Duration::from_secs(2)),
        },
      ],
      dummy_parse,
    );

    {
      let mut state = licker.state.lock().await;
      state.initialized = true;
    }

    let wait = licker.get_wait_for().await;
    assert!(wait.is_some());

    // Should be closer to 2 seconds (minimum) not 10
    let wait_secs = wait.unwrap().as_secs_f64();
    assert!(wait_secs < 5.0, "Expected min wait ~2s, got {:.1}s", wait_secs);
  }

  #[tokio::test]
  async fn get_wait_for_respects_retry_until_as_hard_gate() {
    let now = Instant::now();

    let licker = WindowLicker::new(vec![], dummy_parse);

    {
      let mut state = licker.state.lock().await;
      state.initialized = true;
      state.retry_until = Some(now + Duration::from_secs(5));
    }

    let wait = licker.get_wait_for().await;
    assert!(wait.is_some());
    assert!(wait.unwrap().as_secs() >= 4);
  }

  #[tokio::test]
  async fn get_wait_for_clears_stale_windows() {
    let now = Instant::now();
    let past = now - Duration::from_secs(10);

    let licker = WindowLicker::new(
      vec![Window {
        remaining_key: "r".into(),
        reset_key:     "t".into(),
        remaining:     Some(0),
        reset_at:      Some(past),
      }],
      dummy_parse,
    );

    {
      let mut state = licker.state.lock().await;
      state.initialized = true;
    }

    // Stale window should be cleared, no wait needed
    let wait = licker.get_wait_for().await;
    assert!(wait.is_none());

    // Verify window was cleared
    let state = licker.state.lock().await;
    let window = state.windows.get("r").unwrap();
    assert!(window.remaining.is_none());
    assert!(window.reset_at.is_none());
  }

  #[test]
  fn parse_remaining_handles_comma_separated() {
    let licker = WindowLicker::new(vec![], dummy_parse);
    assert_eq!(licker.parse_remaining("1,000".into()), Some(1000));
    assert_eq!(licker.parse_remaining("  42  ".into()), Some(42));
  }

  #[tokio::test]
  async fn independent_lickers_dont_interfere() {
    let now = Instant::now();

    let licker1 = WindowLicker::new(
      vec![Window {
        remaining_key: "r".into(),
        reset_key:     "t".into(),
        remaining:     Some(0),
        reset_at:      Some(now + Duration::from_secs(10)),
      }],
      dummy_parse,
    );

    let licker2 = WindowLicker::new(
      vec![Window {
        remaining_key: "r".into(),
        reset_key:     "t".into(),
        remaining:     Some(100),
        reset_at:      None,
      }],
      dummy_parse,
    );

    {
      let mut s1 = licker1.state.lock().await;
      let mut s2 = licker2.state.lock().await;
      s1.initialized = true;
      s2.initialized = true;
    }

    // licker1 should need to wait
    let wait1 = licker1.get_wait_for().await;
    assert!(wait1.is_some());

    // licker2 should not need to wait (independent state)
    let wait2 = licker2.get_wait_for().await;
    assert!(wait2.is_none());
  }
}
