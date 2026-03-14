use std::time::Duration;

use crate::consts::JITTER_DIVISOR;
use crate::consts::JITTER_MIN_ADD;
use crate::consts::MAX_BACKOFF_SHIFT;

pub fn compute_retry_delay(base_ms: u64, attempt: u32, max_ms: u64) -> Duration {
  let exp = base_ms.saturating_mul(1u64 << attempt.min(MAX_BACKOFF_SHIFT));
  let cap = exp.min(max_ms.max(base_ms));
  let jitter = rand::random::<u32>() as u64 % (cap / JITTER_DIVISOR + JITTER_MIN_ADD);
  Duration::from_millis(cap / JITTER_DIVISOR + jitter)
}
