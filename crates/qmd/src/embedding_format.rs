pub const DEFAULT_EMBED_MODEL: &str = crate::DEFAULT_EMBED_MODEL_URI;

pub fn is_qwen3_embedding_model(model_uri: &str) -> bool {
  let uri = model_uri.to_ascii_lowercase();
  uri.contains("qwen") && uri.contains("embed")
}

pub fn format_query_for_embedding(query: &str, model_uri: Option<&str>) -> String {
  let env_model = std::env::var("QMD_EMBED_MODEL").ok();
  let uri = model_uri.or(env_model.as_deref()).unwrap_or(DEFAULT_EMBED_MODEL);
  if is_qwen3_embedding_model(uri) {
    return format!("Instruct: Retrieve relevant documents for the given query\nQuery: {query}");
  }
  format!("task: search result | query: {query}")
}

pub fn format_doc_for_embedding(text: &str, title: Option<&str>, model_uri: Option<&str>) -> String {
  let env_model = std::env::var("QMD_EMBED_MODEL").ok();
  let uri = model_uri.or(env_model.as_deref()).unwrap_or(DEFAULT_EMBED_MODEL);
  if is_qwen3_embedding_model(uri) {
    return match title {
      Some(t) => format!("{t}\n{text}"),
      None => text.to_string(),
    };
  }
  format!("title: {} | text: {text}", title.unwrap_or("none"))
}

#[cfg(test)]
mod tests {
  use super::*;

  static ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

  #[test]
  fn qwen_query_format_is_instruct_style() {
    let got = format_query_for_embedding("hello", Some("Qwen/Qwen3-Embedding"));
    assert!(got.starts_with("Instruct: Retrieve relevant documents for the given query\nQuery: "));
  }

  #[test]
  fn non_qwen_query_format_is_task_style() {
    let got = format_query_for_embedding("hello", Some("embeddinggemma"));
    assert_eq!(got, "task: search result | query: hello");
  }

  #[test]
  fn env_var_model_is_used_when_model_uri_is_none() {
    let _guard = ENV_LOCK.lock().unwrap();
    let old = std::env::var("QMD_EMBED_MODEL").ok();
    unsafe { std::env::set_var("QMD_EMBED_MODEL", "qwen-embed") };

    let got = format_query_for_embedding("hello", None);
    assert!(got.contains("Instruct: Retrieve relevant documents"));

    match old {
      Some(v) => unsafe { std::env::set_var("QMD_EMBED_MODEL", v) },
      None => unsafe { std::env::remove_var("QMD_EMBED_MODEL") },
    }
  }
}
