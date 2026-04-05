crate::ix!();

#[derive(Debug)]
pub enum Token {
  String(String),
  Number(String),
  Symbol(char),
  Whitespace,
  Comment,
}

pub fn repair_json_add_missing_quotes(input: &str) -> Result<String, JsonRepairError> {
  let mut changed = false;
  let mut tokens = tokenize(input, &mut changed)?;
  let json_value = parse_value(&mut tokens)?;
  let output = serde_json::to_string(&json_value).map_err(|inner| JsonRepairError::SerdeParseError { inner })?;

  if changed {
    info!("added missing quotations where necessary");
  }

  Ok(output)
}

pub fn tokenize(input: &str, changed: &mut bool) -> Result<VecDeque<Token>, JsonRepairError> {
  let mut tokens = VecDeque::new();
  let mut chars = input.chars().peekable();

  while let Some(&c) = chars.peek() {
    match c {
      '"' | '\'' => {
        let string = parse_quoted_string(&mut chars)?;
        tokens.push_back(Token::String(string));
      }
      '{' | '}' | '[' | ']' | ':' | ',' => {
        chars.next(); // Consume the symbol
        tokens.push_back(Token::Symbol(c));
      }
      '/' if chars.clone().nth(1) == Some('/') => {
        consume_comment(&mut chars);
        tokens.push_back(Token::Comment);
      }
      c if c.is_whitespace() => {
        consume_whitespace(&mut chars);
        tokens.push_back(Token::Whitespace);
      }
      c if c.is_ascii_digit() || c == '-' => {
        let number = parse_number(&mut chars)?;
        tokens.push_back(Token::Number(number));
      }
      _ => {
        let string = parse_unquoted_string(&mut chars)?;
        if !string.is_empty() {
          // We parsed an unquoted string, meaning we "added" quotes logically.
          *changed = true;
        }
        tokens.push_back(Token::String(string));
      }
    }
  }

  Ok(tokens)
}

fn parse_quoted_string(chars: &mut Peekable<Chars>) -> Result<String, JsonRepairError> {
  let quote_char = chars.next().ok_or(JsonRepairError::UnexpectedEOF)?; // opening quote
  let mut s = String::new();

  while let Some(&c) = chars.peek() {
    if c == quote_char {
      chars.next(); // closing quote
      break;
    } else if c == '\\' {
      chars.next(); // consume '\'
      if let Some(escaped_char) = chars.next() {
        s.push(match escaped_char {
          'n' => '\n',
          't' => '\t',
          'r' => '\r',
          'b' => '\x08',
          'f' => '\x0C',
          '\\' => '\\',
          '\'' => '\'',
          '"' => '"',
          other => other,
        });
      } else {
        // If there's nothing after the backslash, treat it as a literal backslash
        s.push('\\');
      }
    } else if ":,{}[]\"'".contains(c) {
      // If we hit a structural character, end the string early
      break;
    } else {
      s.push(chars.next().unwrap());
    }
  }

  Ok(s)
}

fn parse_unquoted_string(chars: &mut Peekable<Chars>) -> Result<String, JsonRepairError> {
  let mut s = String::new();

  while let Some(&c) = chars.peek() {
    if c.is_whitespace() || ":,{}[]\"'".contains(c) {
      break;
    } else {
      s.push(chars.next().unwrap());
    }
  }

  Ok(s.trim().to_string())
}

fn consume_comment(chars: &mut Peekable<Chars>) {
  chars.next(); // '/'
  chars.next(); // second '/'
  for c in chars.clone() {
    if c == '\n' {
      break;
    }
  }
}

fn consume_whitespace(chars: &mut Peekable<Chars>) {
  while let Some(&c) = chars.peek() {
    if c.is_whitespace() {
      chars.next();
    } else {
      break;
    }
  }
}

fn parse_number(chars: &mut Peekable<Chars>) -> Result<String, JsonRepairError> {
  let mut num = String::new();

  while let Some(&c) = chars.peek() {
    if c.is_ascii_digit() || c == '.' || c == 'e' || c == 'E' || c == '+' || c == '-' {
      num.push(chars.next().ok_or(JsonRepairError::UnexpectedEOF)?);
    } else {
      break;
    }
  }

  Ok(num)
}

pub fn unescape_string(s: &str) -> String {
  let mut result = String::new();
  let mut chars = s.chars().peekable();

  while let Some(c) = chars.next() {
    if c == '\\' {
      if let Some(next_char) = chars.next() {
        match next_char {
          'n' => result.push('\n'),
          't' => result.push('\t'),
          'r' => result.push('\r'),
          'b' => result.push('\x08'),
          'f' => result.push('\x0C'),
          '\\' => result.push('\\'),
          '\'' => result.push('\''),
          '"' => result.push('"'),
          other => {
            result.push('\\');
            result.push(other);
          }
        }
      } else {
        result.push('\\');
      }
    } else {
      result.push(c);
    }
  }

  result
}

pub fn parse_value(tokens: &mut VecDeque<Token>) -> Result<JsonValue, JsonRepairError> {
  while let Some(token) = tokens.pop_front() {
    match token {
      Token::Symbol('{') => return parse_object(tokens),
      Token::Symbol('[') => return parse_array(tokens),
      Token::String(s) | Token::Number(s) => {
        let mut value_parts = vec![s];

        loop {
          let continue_loop = {
            let next_token = tokens.front();
            if let Some(next_token) = next_token {
              match next_token {
                Token::Whitespace | Token::Comment => {
                  tokens.pop_front(); // consume and continue
                  true
                }
                Token::String(s) | Token::Number(s) => {
                  let s = s.clone();
                  tokens.pop_front(); // consume
                  value_parts.push(s);
                  true
                }
                _ => false,
              }
            } else {
              false
            }
          };
          if !continue_loop {
            break;
          }
        }

        let s_trimmed = value_parts.join(" ").trim().to_string();
        match s_trimmed.as_str() {
          "true" => return Ok(JsonValue::Bool(true)),
          "false" => return Ok(JsonValue::Bool(false)),
          "null" => return Ok(JsonValue::Null),
          _ => {
            if let Ok(num) = s_trimmed.parse::<i64>() {
              return Ok(JsonValue::Number(num.into()));
            } else if let Ok(num) = s_trimmed.parse::<f64>() {
              if let Some(n) = serde_json::Number::from_f64(num) {
                return Ok(JsonValue::Number(n));
              } else {
                return Err(JsonRepairError::InvalidNumber(s_trimmed.to_string()));
              }
            } else {
              return Ok(JsonValue::String(unescape_string(&s_trimmed)));
            }
          }
        }
      }
      Token::Symbol(c) => {
        if c == ']' || c == '}' {
          if c == ']' {
            return Ok(JsonValue::Array(vec![]));
          } else {
            return Ok(JsonValue::Object(serde_json::Map::new()));
          }
        }
      }
      Token::Whitespace | Token::Comment => continue,
    }
  }
  Ok(JsonValue::Null)
}

pub fn parse_object(tokens: &mut VecDeque<Token>) -> Result<JsonValue, JsonRepairError> {
  if matches!(tokens.front(), Some(Token::Symbol('{'))) {
    tokens.pop_front();
  }

  let mut map = serde_json::Map::new();

  while tokens.front().is_some() {
    while matches!(
      tokens.front(),
      Some(Token::Symbol(',')) | Some(Token::Symbol(':')) | Some(Token::Whitespace) | Some(Token::Comment)
    ) {
      tokens.pop_front();
    }

    match tokens.front() {
      Some(Token::Symbol('}')) => {
        tokens.pop_front(); // consume '}'
        break;
      }
      _ => {
        // Parse key
        let mut key_parts = Vec::new();

        while let Some(token) = tokens.front() {
          match token {
            Token::String(_) | Token::Number(_) => {
              if let Some(token) = tokens.pop_front()
                && let Token::String(s) | Token::Number(s) = token
              {
                key_parts.push(s);
              }
            }
            Token::Whitespace | Token::Comment => {
              tokens.pop_front(); // consume and continue
            }
            _ => break,
          }
        }

        let key = key_parts.join(" ");

        while matches!(tokens.front(), Some(Token::Whitespace) | Some(Token::Comment) | Some(Token::Symbol(','))) {
          tokens.pop_front();
        }

        let colon_found = if let Some(Token::Symbol(':')) = tokens.front() {
          tokens.pop_front(); // consume ':'
          true
        } else {
          false
        };

        while matches!(tokens.front(), Some(Token::Whitespace) | Some(Token::Comment)) {
          tokens.pop_front();
        }

        if colon_found {
          let value = parse_value(tokens)?;
          map.insert(key, value);
        } else {
          match tokens.front() {
            Some(Token::String(_)) | Some(Token::Number(_)) | Some(Token::Symbol('{')) | Some(Token::Symbol('[')) => {
              let value = parse_value(tokens)?;
              map.insert(key, value);
            }
            _ => {
              // No proper value found, treat last key part as value if multiple parts
              if key_parts.len() > 1 {
                let value_str = key_parts.pop().unwrap();
                let key = key_parts.join(" ");
                let value = JsonValue::String(value_str);
                map.insert(key, value);
              } else {
                map.insert(key, JsonValue::Null);
              }
            }
          }
        }
      }
    }
  }

  Ok(JsonValue::Object(map))
}

pub fn parse_array(tokens: &mut VecDeque<Token>) -> Result<JsonValue, JsonRepairError> {
  if let Some(Token::Symbol('[')) = tokens.front() {
    tokens.pop_front();
  }

  let mut arr = vec![];

  while let Some(token) = tokens.front() {
    match token {
      Token::Symbol(']') => {
        tokens.pop_front(); // consume ']'
        break;
      }
      Token::Whitespace | Token::Comment | Token::Symbol(',') => {
        tokens.pop_front(); // consume and continue
        continue;
      }
      _ => {
        let value = parse_value(tokens)?;
        arr.push(value);
      }
    }
  }

  Ok(JsonValue::Array(arr))
}
