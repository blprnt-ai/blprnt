use common::errors::ProviderError;
use common::shared::prelude::ChatRequest;
use common::shared::prelude::LlmModel;
use common::shared::prelude::Provider;

use crate::providers::openai::responses::mapping::OpenAiResponsesMapping;
use crate::providers::openai::responses::request::ResponsesChatRequestBody;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DebugOptions {
  pub echo_upstream_body: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct OpenRouterMapping {
  #[serde(flatten)]
  pub inner:      ResponsesChatRequestBody,
  pub models:     Vec<String>,
  pub plugins:    Vec<OpenRouterPlugin>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub user:       Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub session_id: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub debug:      Option<DebugOptions>,
  pub transforms: Vec<String>,
}

impl OpenRouterMapping {
  pub async fn build_body(
    req: ChatRequest,
    stream: bool,
    tools: Option<serde_json::Value>,
  ) -> std::result::Result<Self, ProviderError> {
    let models = vec![req.llm_model.slug.clone()];

    let inner = OpenAiResponsesMapping::build_body(Provider::OpenRouter, req, stream, tools).await?;

    Ok(Self {
      inner:      inner,
      models:     models,
      plugins:    vec![],
      user:       None,
      session_id: None,
      debug:      None,
      transforms: vec!["middle-out".to_string()],
    })
  }

  pub fn build_body_basic(model: LlmModel, prompt: String, system: String) -> Self {
    let models = vec![model.slug.clone()];

    let inner = OpenAiResponsesMapping::build_body_basic(model.slug, model.supports_reasoning, prompt, system);

    Self {
      inner:      inner,
      models:     models,
      plugins:    vec![],
      user:       None,
      session_id: None,
      debug:      None,
      transforms: vec!["middle-out".to_string()],
    }
  }

  #[allow(dead_code)]
  pub fn with_user(mut self, user: String) -> Self {
    self.user = Some(user);
    self
  }

  #[allow(dead_code)]
  pub fn with_session_id(mut self, session_id: String) -> Self {
    self.session_id = Some(session_id);
    self
  }

  pub fn with_debug(mut self, echo_upstream_body: bool) -> Self {
    self.debug = Some(DebugOptions { echo_upstream_body });
    self
  }

  // TODO: Implement this
  #[allow(dead_code)]
  pub fn with_web(
    self,
    max_results: Option<u32>,
    search_prompt: Option<String>,
    engine: Option<OpenRouterWebEngine>,
  ) -> Self {
    self.add_plugin(OpenRouterPlugin::Web(OpenRouterWebPlugin { max_results, search_prompt, engine }))
  }

  #[allow(dead_code)]
  pub fn with_file_parser(self, pdf: OpenRouterFileParserPdfEngine) -> Self {
    self.add_plugin(pdf.into())
  }

  #[allow(dead_code)]
  pub fn with_moderation(self) -> Self {
    self.add_plugin(OpenRouterPlugin::Moderation)
  }

  #[allow(dead_code)]
  pub fn with_response_healing(self) -> Self {
    self.add_plugin(OpenRouterPlugin::ResponseHealing)
  }

  fn add_plugin(mut self, plugin: OpenRouterPlugin) -> Self {
    self.plugins.push(plugin);
    self
  }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "id", rename_all = "kebab-case")]
pub enum OpenRouterPlugin {
  Moderation,
  Web(OpenRouterWebPlugin),
  FileParser(OpenRouterFileParserPlugin),
  ResponseHealing,
  AutoRouter(OpenRouterAutoRouterPlugin),
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct OpenRouterWebPlugin {
  #[serde(skip_serializing_if = "Option::is_none")]
  pub max_results:   Option<u32>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub search_prompt: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub engine:        Option<OpenRouterWebEngine>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum OpenRouterWebEngine {
  Native,
  Exa,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct OpenRouterFileParserPlugin {
  pub pdf: OpenRouterFileParserPdf,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct OpenRouterFileParserPdf {
  pub engine: OpenRouterFileParserPdfEngine,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum OpenRouterFileParserPdfEngine {
  MistralOcr,
  PdfText,
  Native,
}

impl From<OpenRouterFileParserPdfEngine> for OpenRouterPlugin {
  fn from(engine: OpenRouterFileParserPdfEngine) -> Self {
    OpenRouterPlugin::FileParser(OpenRouterFileParserPlugin { pdf: OpenRouterFileParserPdf { engine } })
  }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct OpenRouterAutoRouterPlugin {
  pub allowed_models: Vec<String>,
}

impl From<Vec<String>> for OpenRouterPlugin {
  fn from(allowed_models: Vec<String>) -> Self {
    OpenRouterPlugin::AutoRouter(OpenRouterAutoRouterPlugin { allowed_models })
  }
}
