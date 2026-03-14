use axum::http::HeaderValue;
use common::blprnt::Blprnt;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;
use tauri::Emitter;

pub const PREVIEW_INSTRUMENTATION_EVENT_NAME: &str = "previewInstrumentation";
pub const PREVIEW_INSTRUMENTATION_SCRIPT_PATH: &str = "/__blprnt/preview/instrumentation.js";
pub const PREVIEW_INSTRUMENTATION_EVENT_PATH: &str = "/__blprnt/preview/event";

const SCRIPT_TAG: &str =
  "<script data-blprnt-preview-instrumentation src=\"/__blprnt/preview/instrumentation.js\"></script>";

#[derive(Clone, Debug)]
pub struct PreviewInstrumentationConfig {
  pub enabled:    bool,
  pub session_id: String,
  pub project_id: String,
}

impl PreviewInstrumentationConfig {
  pub fn enabled(session_id: String, project_id: String) -> Self {
    Self { enabled: true, session_id, project_id }
  }
}

#[derive(Debug, Deserialize)]
pub struct PreviewInstrumentationEventRequest {
  #[serde(rename = "type")]
  pub event_type: String,
  #[serde(default)]
  pub payload:    Value,
  pub timestamp:  Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviewInstrumentationEventPayload {
  pub session_id: String,
  pub project_id: String,
  pub event_type: String,
  pub payload:    Value,
  pub timestamp:  Option<String>,
}

pub fn instrumentation_script() -> &'static str {
  r#"(() => {
  if (window.__blprntPreviewInstrumentation) return;
  window.__blprntPreviewInstrumentation = true;

  const endpoint = '/__blprnt/preview/event';

  const send = (type, payload) => {
    const body = JSON.stringify({ type, payload, timestamp: new Date().toISOString() });
    try {
      if (navigator.sendBeacon) {
        const blob = new Blob([body], { type: 'application/json' });
        navigator.sendBeacon(endpoint, blob);
        return;
      }
    } catch (_) {}

    fetch(endpoint, {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body,
      keepalive: true,
      credentials: 'omit',
    }).catch(() => {});
  };

  const pageReady = () => {
    send('page-ready', { url: window.location.href, title: document.title });
  };

  if (document.readyState === 'loading') {
    window.addEventListener('DOMContentLoaded', pageReady, { once: true });
  } else {
    setTimeout(pageReady, 0);
  }

  const emitRoute = (reason) => {
    send('route-change', { url: window.location.href, reason });
  };

  const wrapHistory = (method) => {
    const original = history[method];
    if (!original) return;
    history[method] = function (...args) {
      const result = original.apply(this, args);
      emitRoute(method);
      return result;
    };
  };

  wrapHistory('pushState');
  wrapHistory('replaceState');
  window.addEventListener('popstate', () => emitRoute('popstate'));
  window.addEventListener('hashchange', () => emitRoute('hashchange'));

  const reportError = (details) => {
    send('network-error', { ...details, url: window.location.href });
  };

  if (window.fetch) {
    const originalFetch = window.fetch;
    window.fetch = function (...args) {
      return originalFetch.apply(this, args).then((response) => {
        if (!response.ok) {
          reportError({ kind: 'fetch', message: `HTTP ${response.status}`, status: response.status, statusText: response.statusText });
        }
        return response;
      }).catch((error) => {
        reportError({ kind: 'fetch', message: error?.message ?? String(error) });
        throw error;
      });
    };
  }

  const originalXhrOpen = XMLHttpRequest.prototype.open;
  const originalXhrSend = XMLHttpRequest.prototype.send;

  XMLHttpRequest.prototype.open = function (method, url, ...rest) {
    this.__blprntRequestUrl = url;
    return originalXhrOpen.call(this, method, url, ...rest);
  };

  XMLHttpRequest.prototype.send = function (...args) {
    this.addEventListener('error', () => {
      reportError({ kind: 'xhr', message: 'XMLHttpRequest error', requestUrl: this.__blprntRequestUrl });
    });
    this.addEventListener('abort', () => {
      reportError({ kind: 'xhr', message: 'XMLHttpRequest aborted', requestUrl: this.__blprntRequestUrl });
    });
    this.addEventListener('load', () => {
      if (this.status >= 400) {
        reportError({
          kind: 'xhr',
          message: `HTTP ${this.status}`,
          status: this.status,
          statusText: this.statusText,
          requestUrl: this.__blprntRequestUrl,
        });
      }
    });
    return originalXhrSend.apply(this, args);
  };

  window.addEventListener(
    'error',
    (event) => {
      const target = event.target;
      if (target && target !== window && target.tagName) {
        reportError({
          kind: 'resource',
          message: 'Resource load error',
          tag: target.tagName,
          src: target.src || target.href || null,
        });
      }
    },
    true,
  );

  window.addEventListener(
    'unhandledrejection',
    (event) => {
      const reason = event.reason;
      reportError({
        kind: 'unhandledrejection',
        message: reason?.message ?? String(reason),
        stack: reason?.stack,
      });
    },
    true,
  );
})();
"#
}

pub fn inject_instrumentation(html: &str) -> String {
  if html.contains("data-blprnt-preview-instrumentation") {
    return html.to_string();
  }

  if let Some(index) = html.rfind("</head>") {
    let mut injected = String::with_capacity(html.len() + SCRIPT_TAG.len());
    injected.push_str(&html[..index]);
    injected.push_str(SCRIPT_TAG);
    injected.push_str(&html[index..]);
    return injected;
  }

  if let Some(index) = html.rfind("</body>") {
    let mut injected = String::with_capacity(html.len() + SCRIPT_TAG.len());
    injected.push_str(&html[..index]);
    injected.push_str(SCRIPT_TAG);
    injected.push_str(&html[index..]);
    return injected;
  }

  format!("{}{}", html, SCRIPT_TAG)
}

pub fn is_html_response(content_type: Option<&HeaderValue>) -> bool {
  content_type
    .and_then(|value| value.to_str().ok())
    .map(|value| value.to_ascii_lowercase().contains("text/html"))
    .unwrap_or(false)
}

pub fn emit_instrumentation_event(config: &PreviewInstrumentationConfig, event: PreviewInstrumentationEventRequest) {
  let payload = PreviewInstrumentationEventPayload {
    session_id: config.session_id.clone(),
    project_id: config.project_id.clone(),
    event_type: event.event_type,
    payload:    event.payload,
    timestamp:  event.timestamp,
  };

  tracing::debug!(
    session_id = payload.session_id,
    project_id = payload.project_id,
    event_type = payload.event_type,
    "Preview instrumentation event"
  );

  let handle = Blprnt::handle();
  let _ = handle.emit(PREVIEW_INSTRUMENTATION_EVENT_NAME, payload);
}
