use serde_json::Value;
#[derive(Clone, Debug)]
pub struct ToolSchemaRegistry {
  pub(crate) schemas: Value,
}

impl ToolSchemaRegistry {
  pub fn new(schemas: Value) -> Self {
    Self { schemas }
  }

  pub fn schemas(&self) -> &Value {
    &self.schemas
  }
}
