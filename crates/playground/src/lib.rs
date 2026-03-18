use serde_json::Value;

#[derive(Debug, serde::Serialize)]
pub struct User {
  pub id:      String,
  pub details: Value,
}

#[cfg(test)]
mod tests {

  use super::*;

  #[test]
  fn test_serialize() {
    let user = User {
      id:      "123".to_string(),
      details: serde_json::json!({
        "name": "John Doe",
        "age": 30,
      }),
    };

    let json = serde_json::to_string(&user).unwrap();
    println!("json: {}", json);
  }
}
