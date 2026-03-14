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
use reqwest_middleware::Result;

use crate::util::window_licker::Window;
use crate::util::window_licker::WindowLicker;

/// Registry of OpenAI HTTP clients, keyed by API key hash.
/// Each unique API key gets its own client with independent rate limiting.
static CLIENT_REGISTRY: once_cell::sync::Lazy<RwLock<HashMap<u64, Arc<OpenAiHttp>>>> =
  once_cell::sync::Lazy::new(|| RwLock::new(HashMap::new()));

#[derive(Clone, Debug)]
pub struct OpenAiHttp {
  pub client: ClientWithMiddleware,
}

impl OpenAiHttp {
  /// Get or create a client for the given API key.
  /// Clients are cached by API key hash, so same-key requests share rate limiting.
  pub fn get_client(api_key: &str) -> Arc<OpenAiHttp> {
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

    let middleware = OpenAiMiddleware::new();
    let client = ClientBuilder::new(reqwest::Client::new()).with(middleware).build();
    let http = Arc::new(OpenAiHttp { client });
    registry.insert(key_hash, Arc::clone(&http));
    http
  }
}

#[derive(Clone, Debug)]
struct OpenAiMiddleware {
  window_licker: WindowLicker,
}

fn parse_openai_header(string: String) -> Option<time::Duration> {
  let duration = humantime::parse_duration(&string).ok()?;
  let duration = time::Duration::nanoseconds(duration.as_nanos() as i64);

  Some(duration)
}

impl OpenAiMiddleware {
  pub fn new() -> Self {
    let windows = vec![
      Window::new("x-ratelimit-remaining-requests".to_string(), "x-ratelimit-reset-requests".to_string()),
      Window::new("x-ratelimit-remaining-tokens".to_string(), "x-ratelimit-reset-tokens".to_string()),
    ];
    let window_licker = WindowLicker::new(windows, parse_openai_header);
    Self { window_licker }
  }
}

#[async_trait::async_trait]
impl Middleware for OpenAiMiddleware {
  async fn handle(&self, req: Request, extensions: &mut Extensions, next: Next<'_>) -> Result<Response> {
    self.window_licker.preflight().await;
    let resp = next.run(req, extensions).await?;
    let _ = self.window_licker.update_from_response(&resp).await;
    Ok(resp)
  }
}
