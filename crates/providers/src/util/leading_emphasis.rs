pub fn strip_leading_emphasis(input: &str) -> (String, Option<String>) {
  if let Some(rest) = input.strip_prefix('*')
    && !rest.starts_with('*')
  {
    let mut idx = 0usize;
    let bytes = rest.as_bytes();
    while idx < bytes.len() {
      if bytes[idx] == b'*' && (idx == 0 || bytes[idx - 1] != b'\\') {
        let captured = rest[..idx].to_string();
        let remainder = &rest[idx + 1..];
        return (remainder.to_string(), Some(captured));
      }
      idx += 1;
    }
  }
  (input.to_string(), None)
}
