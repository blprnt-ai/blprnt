use std::net::SocketAddr;

use axum::Router;
use axum::body::Body;
use axum::extract::Request;
use axum::extract::connect_info::ConnectInfo;
use axum::http::StatusCode;
use axum::http::header;
use axum::middleware;
use axum::middleware::Next;
use axum::response::Response;
use http_body_util::BodyExt;

use crate::preview::instrumentation::PreviewInstrumentationConfig;
use crate::preview::instrumentation::inject_instrumentation;
use crate::preview::instrumentation::is_html_response;
use crate::preview::proxy::instrumentation_router;
use crate::preview::proxy::static_router;

pub fn wrap_static_router(router: Router) -> Router {
  router.layer(middleware::from_fn(static_request_guard))
}

pub fn static_app(static_path: &str, instrumentation: PreviewInstrumentationConfig) -> Router {
  let router = static_router(static_path);
  let router =
    if instrumentation.enabled { router.merge(instrumentation_router(instrumentation.clone())) } else { router };
  let router = if instrumentation.enabled {
    router.layer(middleware::from_fn(move |req: Request, next: Next| {
      let instrumentation = instrumentation.clone();
      async move {
        let response = next.run(req).await;
        Ok::<_, StatusCode>(inject_response_if_html(response, &instrumentation).await)
      }
    }))
  } else {
    router
  };
  wrap_static_router(router)
}

async fn static_request_guard(
  ConnectInfo(remote_addr): ConnectInfo<SocketAddr>,
  request: Request,
  next: Next,
) -> Result<Response, StatusCode> {
  if !remote_addr.ip().is_loopback() {
    return Err(StatusCode::FORBIDDEN);
  }

  Ok(next.run(request).await)
}

async fn inject_response_if_html(response: Response, instrumentation: &PreviewInstrumentationConfig) -> Response {
  if !instrumentation.enabled {
    return response;
  }

  if !is_html_response(response.headers().get(header::CONTENT_TYPE)) {
    return response;
  }

  let (parts, body) = response.into_parts();
  let (parts, body_str) = match collect_body_string(parts, body).await {
    Ok(value) => value,
    Err(parts) => return Response::from_parts(parts, Body::empty()),
  };

  let mut parts = parts;
  parts.headers.remove(header::CONTENT_LENGTH);
  Response::from_parts(parts, Body::from(inject_instrumentation(&body_str)))
}

async fn collect_body_string(
  parts: axum::http::response::Parts,
  body: Body,
) -> Result<(axum::http::response::Parts, String), axum::http::response::Parts> {
  let collected = match body.collect().await {
    Ok(value) => value,
    Err(_) => return Err(parts),
  };

  match String::from_utf8(collected.to_bytes().to_vec()) {
    Ok(body_str) => Ok((parts, body_str)),
    Err(_) => Err(parts),
  }
}
