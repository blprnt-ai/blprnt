use common::errors::ProviderError;
use regex::Regex;
use serde_json::Value;
use serde_json::json;

use crate::tools::adapter::ToolSchemaAdapter;

const MAX_TOOL_NAME_LEN: usize = 64;
pub struct AnthropicToolAdapter {
  name_re:       Regex,
  underscore_re: Regex,
}

impl ToolSchemaAdapter for AnthropicToolAdapter {
  fn adapt(&self, input: &str) -> anyhow::Result<String> {
    let v: Value = serde_json::from_str(input)?;
    let schemas = self.extract_schemas(v)?;
    let tools: anyhow::Result<Vec<_>> = schemas.into_iter().map(|s| self.schema_to_tool(s)).collect();

    Ok(serde_json::to_string(&json!({ "tools": tools? }))?)
  }
}

impl AnthropicToolAdapter {
  fn schema_to_tool(&self, mut schema: Value) -> anyhow::Result<Value> {
    let (name, desc) = self.derive_name_and_description(schema.get("$id"))?;
    let mut schema_obj = if let Some(o) = schema.as_object_mut() {
      o.remove("$id");
      serde_json::to_value(o.clone()).map_err(|e| ProviderError::InvalidSchema(e.to_string()))?
    } else {
      return Err(ProviderError::InvalidSchema("schema must be object".into()).into());
    };

    if let Some(all_of) = schema_obj.get_mut("allOf").map(|v| v.take()).and_then(|v| v.as_array().cloned()) {
      let mut branches: Vec<Value> = Vec::new();
      for item in all_of {
        let action = item
          .get("if")
          .and_then(|iff| iff.get("properties"))
          .and_then(|p| p.get("action"))
          .and_then(|a| a.get("const"))
          .and_then(|c| c.as_str())
          .map(|s| s.to_string());
        let reqs: Option<Vec<String>> = item
          .get("then")
          .and_then(|t| t.get("required"))
          .and_then(|r| r.as_array())
          .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect());
        if let (Some(act), Some(required)) = (action, reqs) {
          branches.push(json!({
            "properties": { "action": { "const": act } },
            "required": required,
          }));
        }
      }

      if !branches.is_empty()
        && let Some(obj) = schema_obj.as_object_mut()
      {
        obj.remove("allOf");
        obj.insert("oneOf".into(), Value::Array(branches));
      }
    }

    if let Some(obj) = schema_obj.as_object_mut() {
      obj.entry("additionalProperties").or_insert(Value::Bool(false));
      if let Some(props) = obj.get_mut("properties").and_then(|p| p.as_object_mut()) {
        if let Some(action) = props.get_mut("action").and_then(|a| a.as_object_mut()) {
          action.entry("type").or_insert(Value::String("string".into()));
        }

        if let Some(coll) = props.get_mut("collection").and_then(|c| c.as_object_mut()) {
          coll.entry("type").or_insert(Value::String("string".into()));
        }
      }
    }

    let description = schema_obj.get("description").and_then(|d| d.as_str()).or(desc.as_deref());
    let mut tool = json!({ "name": name, "input_schema": schema_obj });
    if let Some(d) = description {
      tool.as_object_mut().unwrap().insert("description".into(), Value::String(d.to_string()));
    }

    Ok(tool)
  }

  fn derive_name_and_description(&self, id_value: Option<&Value>) -> anyhow::Result<(String, Option<String>)> {
    let mut name = "unnamed_tool".to_string();
    let mut description = None;
    if let Some(Value::String(id)) = id_value {
      if let Some(c) = self.name_re.captures(id) {
        let base = c.get(1).unwrap().as_str().to_lowercase();
        let ver = c.get(2).unwrap().as_str();
        name = format!("{}_v{}", self.sanitize_name(&base), ver);
      } else {
        let tail = id.rsplit('.').next().unwrap_or(id);
        name = self.sanitize_name(tail);
      }

      description = Some(format!("Function derived from schema id: {}", id));
    }

    if name.len() > MAX_TOOL_NAME_LEN {
      name.truncate(MAX_TOOL_NAME_LEN);
    }

    Ok((name, description))
  }

  fn sanitize_name(&self, s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
      out.push(if ch.is_ascii_alphanumeric() { ch.to_ascii_lowercase() } else { '_' });
    }

    self.underscore_re.replace_all(&out, "_").trim_matches('_').to_string()
  }
}

impl Default for AnthropicToolAdapter {
  fn default() -> Self {
    Self {
      name_re:       Regex::new(r"(?i)^tool\.([a-z0-9_]+)\.v(\d+)$").unwrap(),
      underscore_re: Regex::new(r"_+").unwrap(),
    }
  }
}
