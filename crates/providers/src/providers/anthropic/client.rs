use std::collections::HashMap;
use std::hash::DefaultHasher;
use std::hash::Hash;
use std::hash::Hasher;
use std::sync::Arc;

use http::Extensions;
use parking_lot::RwLock;
use reqwest::Request;
use reqwest::Response;
use reqwest_middleware::ClientBuilder;
use reqwest_middleware::ClientWithMiddleware;
use reqwest_middleware::Middleware;
use reqwest_middleware::Next;
use reqwest_middleware::Result as MwResult;
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

use crate::util::window_licker::Window;
use crate::util::window_licker::WindowLicker;

/// Registry of Anthropic HTTP clients, keyed by API key hash.
/// Each unique API key gets its own client with independent rate limiting.
static CLIENT_REGISTRY: once_cell::sync::Lazy<RwLock<HashMap<u64, Arc<AnthropicHttp>>>> =
  once_cell::sync::Lazy::new(|| RwLock::new(HashMap::new()));

#[derive(Clone, Debug)]
pub struct AnthropicHttp {
  pub client: ClientWithMiddleware,
}

impl AnthropicHttp {
  /// Get or create a client for the given API key.
  /// Clients are cached by API key hash, so same-key requests share rate limiting.
  pub fn get_client(api_key: &str) -> Arc<AnthropicHttp> {
    let key_hash = {
      let mut hasher = DefaultHasher::new();
      api_key.hash(&mut hasher);
      hasher.finish()
    };

    // Fast path: check if client exists
    {
      let registry = CLIENT_REGISTRY.read();
      if let Some(client) = registry.get(&key_hash) {
        return Arc::clone(client);
      }
    }

    // Slow path: create new client
    let mut registry = CLIENT_REGISTRY.write();
    // Double-check after acquiring write lock
    if let Some(client) = registry.get(&key_hash) {
      return Arc::clone(client);
    }

    let middleware = AnthropicMiddleware::new();
    let client = ClientBuilder::new(reqwest::Client::new()).with(middleware).build();
    let http = Arc::new(AnthropicHttp { client });
    registry.insert(key_hash, Arc::clone(&http));
    http
  }
}

#[derive(Clone, Debug)]
struct AnthropicMiddleware {
  window_licker: WindowLicker,
}

fn parse_anthropic_header(string: String) -> Option<time::Duration> {
  let time = OffsetDateTime::parse(string.trim(), &Rfc3339).ok()?;
  let now = OffsetDateTime::now_utc();
  let duration = (time - now).max(time::Duration::ZERO);

  Some(duration)
}

impl AnthropicMiddleware {
  pub fn new() -> Self {
    let windows = vec![
      Window::new(
        "anthropic-ratelimit-requests-remaining".to_string(),
        "anthropic-ratelimit-requests-reset".to_string(),
      ),
      Window::new("anthropic-ratelimit-tokens-remaining".to_string(), "anthropic-ratelimit-tokens-reset".to_string()),
      Window::new(
        "anthropic-ratelimit-input-tokens-remaining".to_string(),
        "anthropic-ratelimit-input-tokens-reset".to_string(),
      ),
      Window::new(
        "anthropic-ratelimit-output-tokens-remaining".to_string(),
        "anthropic-ratelimit-output-tokens-reset".to_string(),
      ),
    ];
    let window_licker = WindowLicker::new(windows, parse_anthropic_header);

    Self { window_licker }
  }
}

#[async_trait::async_trait]
impl Middleware for AnthropicMiddleware {
  async fn handle(&self, req: Request, extensions: &mut Extensions, next: Next<'_>) -> MwResult<Response> {
    self.window_licker.preflight().await;
    let resp = next.run(req, extensions).await?;
    self.window_licker.update_from_response(&resp).await;
    Ok(resp)
  }
}
