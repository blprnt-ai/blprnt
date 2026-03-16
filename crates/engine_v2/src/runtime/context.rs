#![allow(clippy::bind_instead_of_map)]

use std::collections::HashMap;
use std::collections::HashSet;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use anyhow::Result;
use common::agent::ToolAllowList;
use common::blprnt::Blprnt;
use common::blprnt_settings::ADVANCED_REASONING_EFFORT_CLASSIFIER_ENABLED_KEY;
use common::blprnt_settings::ADVANCED_SKILL_MATCHER_ENABLED_KEY;
use common::blprnt_settings::default_advanced_pre_turn_helper_enabled;
use common::blprnt_settings::store_bool_with_default_true;
use common::errors::AppCoreError;
use common::errors::EngineError;
use common::memory::ManagedMemoryStore;
use common::memory::QmdMemorySearchService;
use common::models::ReasoningEffort;
use common::paths::BlprntPath;
use common::personality_service::PersonalityService;
use common::plan_utils::get_plan_content_by_parent_session_id;
use common::session_dispatch::SessionDispatch;
use common::shared::prelude::BlprntCredentials;
use common::shared::prelude::ChatRequest;
use common::shared::prelude::LlmModel;
use common::shared::prelude::McpToolDescriptor;
use common::shared::prelude::OauthToken;
use common::shared::prelude::PlanContext;
use common::shared::prelude::Provider;
use common::shared::prelude::SurrealId;
use common::shared::prelude::mcp_tool_runtime_name;
use common::skills_utils::SkillsUtils;
use common::tools::MemorySearchRequest;
use common::tools::config::ToolsSchemaConfig;
use oauth::anthropic::oauth::AnthropicOauth;
use oauth::consts as oauth_consts;
use oauth::openai::oauth::OpenAiOauth;
use persistence::prelude::ProviderRecord;
use persistence::prelude::ProviderRepositoryV2;
use persistence::prelude::SessionRecord;
use persistence::prelude::SessionRepositoryV2;
use providers::ProviderAdapter;
use providers::build_adapter;
use providers::providers::mock::provider::MockProvider;
use providers::tools::registry::ToolSchemaRegistry;
use serde_json;
use session::Session;
use surrealdb::types::ToSql;
use surrealdb::types::Uuid;
use tauri_plugin_store::StoreExt;
use tokio::sync::RwLock;
use tokio_util::sync::CancellationToken;
use tools::Tool;
use tools::Tools;
use vault::Vault;
use vault::get_stronghold_secret;
use vault::set_stronghold_secret;

const USE_MOCK_PROVIDER: bool = false;
const OAUTH_REFRESH_SKEW_MS: u64 = 5 * 60 * 1000;
const OPENAI_OAUTH_MAX_TTL_MS: u64 = 60 * 60 * 1000;
const BLPRNT_STORE: &str = "blprnt.json";

use crate::hooks::prelude::*;
use crate::prelude::ControllerConfig;

#[derive(Clone, Debug)]
pub(crate) struct RuntimeContext {
  pub user_prompt:          String,
  pub session_id:           SurrealId,
  pub project_id:           SurrealId,
  pub reasoning_effort:     Arc<RwLock<Option<ReasoningEffort>>>,
  pub current_skills:       Arc<RwLock<Option<Vec<String>>>>,
  pub current_prompt:       Arc<RwLock<Option<String>>>,
  pub session_dispatch:     Arc<SessionDispatch>,
  pub cancel_token:         CancellationToken,
  pub provider_adapter:     Arc<ProviderAdapter>,
  pub tools_registry:       Arc<ToolSchemaRegistry>,
  pub mcp_runtime:          Option<common::shared::prelude::McpRuntimeBridgeRef>,
  pub hook_registry:        Arc<HookRegistry>,
  pub is_subagent:          bool,
  pub memory_tools_enabled: bool,
  pub mcp_details:          HashMap<String, String>,
  pub cached_memory:        Arc<RwLock<Option<String>>>,
}

impl RuntimeContext {
  pub async fn new(
    config: ControllerConfig,
    session_dispatch: Arc<SessionDispatch>,
    cancel_token: CancellationToken,
    session_id: SurrealId,
    is_subagent: bool,
    user_prompt: String,
  ) -> Result<RuntimeContext> {
    // Session should already be initialized by this point
    let session_model = SessionRepositoryV2::get(session_id.clone()).await?;
    let project_id = session_model.project.clone();

    let working_directories = Session::working_directories(&project_id).await?;
    let enabled_models = Self::visible_enabled_models().await?;

    let model = Self::llm_model(&session_model.clone()).await?;

    let provider_adapter = Self::provider_adapter(model).await?;

    let mut tools_schema = Tools::schema(&ToolsSchemaConfig {
      agent_kind:           *session_model.agent_kind(),
      working_directories:  working_directories.into(),
      is_subagent:          config.is_subagent,
      memory_tools_enabled: config.memory_tools_enabled,
      enabled_models:       enabled_models.clone(),
    });

    let mut mcp_details = HashMap::new();
    if let Some(mcp_runtime) = config.mcp_runtime.clone() {
      let mcp_tools = mcp_runtime.list_tools().await;
      tools_schema.extend(Self::mcp_tool_specs(mcp_tools, *session_model.agent_kind(), config.is_subagent));
      mcp_details = mcp_runtime
        .get_initialize_results()
        .await
        .into_iter()
        .filter_map(|(k, v)| v.instructions.map(|i| (k, i)))
        .collect();
    }

    let tools_registry = ToolSchemaRegistry::new(serde_json::json!(tools_schema));
    let hook_registry = Self::build_hook_registry();

    Ok(RuntimeContext {
      user_prompt:          user_prompt,
      session_id:           session_id,
      project_id:           project_id,
      reasoning_effort:     Arc::new(RwLock::new(None)),
      current_skills:       Arc::new(RwLock::new(None)),
      current_prompt:       Arc::new(RwLock::new(None)),
      session_dispatch:     session_dispatch,
      cancel_token:         cancel_token,
      provider_adapter:     provider_adapter,
      tools_registry:       Arc::new(tools_registry),
      mcp_runtime:          config.mcp_runtime.clone(),
      hook_registry:        hook_registry,
      is_subagent:          is_subagent,
      memory_tools_enabled: config.memory_tools_enabled,
      mcp_details:          mcp_details,
      cached_memory:        Arc::new(RwLock::new(None)),
    })
  }

  pub async fn set_reasoning_effort(&self, reasoning_effort: ReasoningEffort) {
    let mut guard = self.reasoning_effort.write().await;
    *guard = Some(reasoning_effort);
  }

  pub async fn reasoning_effort(&self) -> Option<ReasoningEffort> {
    let guard = self.reasoning_effort.read().await;
    *guard
  }

  pub async fn set_current_prompt(&self, prompt: String) {
    let mut guard = self.current_prompt.write().await;
    *guard = Some(prompt);
  }

  pub async fn current_prompt(&self) -> Option<String> {
    let guard = self.current_prompt.read().await;
    guard.clone()
  }

  pub async fn set_current_skills(&self, skills: Vec<String>) {
    let mut guard = self.current_skills.write().await;
    *guard = Some(skills);
  }

  pub async fn current_skills(&self) -> Option<Vec<String>> {
    let guard = self.current_skills.read().await;
    guard.clone()
  }

  pub async fn build_chat_request(&self) -> Result<ChatRequest> {
    let session_model = SessionRepositoryV2::get(self.session_id.clone()).await?;
    let model = Self::llm_model(&session_model).await?;

    let personality = Self::resolve_personality_prompt(&session_model)?;

    let current_skills = self
      .current_skills()
      .await
      .unwrap_or_default()
      .into_iter()
      .filter_map(|skill| SkillsUtils::get_skill_content(&skill))
      .collect::<Vec<_>>();

    let memory_guard = self.cached_memory.read().await;
    let memory = memory_guard.as_ref().cloned();
    drop(memory_guard);

    let memory = match memory {
      Some(memory) => memory.clone(),
      None => {
        let memory_summary = ManagedMemoryStore::memory_for_agent(&BlprntPath::memories_root()).await;

        let memory = match QmdMemorySearchService::new(self.project_id.key().to_string())
          .search(&MemorySearchRequest { query: self.user_prompt.clone(), limit: Some(5) }, Some(0.35))
          .await
          .and_then(|result| Ok(result.memories.into_iter().map(|m| m.content).collect::<Vec<_>>().join("\n\n")))
          .and_then(|memory| Ok(format!("The following memories may be relevant to the user's request. If they're not, ignore them.\n<memories>\n{memory}\n</memories>")))
          .ok()
        {
          Some(memory) => format!("{memory_summary}\n{memory}"),
          None => memory_summary,
        };

        *self.cached_memory.write().await = Some(memory.clone());
        memory
      }
    };

    Ok(ChatRequest {
      agent_kind:          *session_model.agent_kind(),
      agent_primer:        Session::agent_primer(&self.project_id).await?,
      instructions:        None,
      llm_model:           model,
      personality:         personality,
      reasoning_effort:    self.reasoning_effort.read().await.unwrap_or(ReasoningEffort::Minimal),
      session_id:          self.session_id.clone(),
      working_directories: Session::working_directories(&self.project_id).await?,
      current_skills:      current_skills,
      plan_context:        self.build_plan_context(&session_model),
      mcp_details:         self.mcp_details.clone(),
      memory:              memory,
    })
  }

  fn resolve_personality_prompt(session_model: &SessionRecord) -> Result<String> {
    let service = PersonalityService::new();

    if let Some(personality_key) = session_model.personality_key().as_ref()
      && let Some(selected) = service.get(personality_key)?
    {
      return Ok(selected.body);
    }

    Ok(String::new())
  }

  fn build_plan_context(&self, session_model: &SessionRecord) -> Option<PlanContext> {
    let parent_session_id =
      session_model.parent_id.as_ref().map(|id| id.0.to_sql()).unwrap_or_else(|| session_model.id.0.to_sql());

    if let Ok(Some(plan)) = get_plan_content_by_parent_session_id(session_model.project.clone(), &parent_session_id) {
      return Some(PlanContext { id: plan.id, content: plan.content });
    }

    let _ = session_model;
    None
  }

  fn build_hook_registry() -> Arc<HookRegistry> {
    let mut hook_registry = HookRegistry::new();
    let (reasoning_effort_classifier_enabled, skill_matcher_enabled) = Self::advanced_pre_turn_hook_settings();

    // OnPreTurn
    hook_registry.register_hook(HookKind::PreTurn, Box::new(StartTurn));
    hook_registry.register_hook(HookKind::PreTurn, Box::new(MaybeHealOrphans));
    if reasoning_effort_classifier_enabled {
      hook_registry.register_hook(HookKind::PreTurn, Box::new(ReasoningEffortClassifier));
    }
    if skill_matcher_enabled {
      hook_registry.register_hook(HookKind::PreTurn, Box::new(SkillMatcherHook));
    }

    // OnPreStep
    hook_registry.register_hook(HookKind::PreStep, Box::new(SessionTokenUsage));

    // OnPostStep
    // Nothing here yet

    // OnPostTurn
    hook_registry.register_hook(HookKind::PostTurn, Box::new(EndTurn));

    Arc::new(hook_registry)
  }

  fn advanced_pre_turn_hook_settings() -> (bool, bool) {
    let Ok(store) = Blprnt::handle().store(BLPRNT_STORE) else {
      return (default_advanced_pre_turn_helper_enabled(), default_advanced_pre_turn_helper_enabled());
    };

    (
      store_bool_with_default_true(store.get(ADVANCED_REASONING_EFFORT_CLASSIFIER_ENABLED_KEY).as_ref()),
      store_bool_with_default_true(store.get(ADVANCED_SKILL_MATCHER_ENABLED_KEY).as_ref()),
    )
  }

  fn mcp_tool_specs(
    tools: Vec<McpToolDescriptor>,
    agent_kind: common::agent::AgentKind,
    is_subagent: bool,
  ) -> Vec<common::tools::ToolSpec> {
    tools
      .into_iter()
      .filter_map(|tool| {
        let runtime_name = mcp_tool_runtime_name(&tool.server_id, &tool.name);
        if !ToolAllowList::is_tool_allowed_and_enabled(
          common::agent::ToolId::Mcp(runtime_name.clone()),
          agent_kind,
          is_subagent,
        ) {
          return None;
        }
        let schema = if tool.input_schema.is_object() {
          tool.input_schema
        } else {
          serde_json::json!({ "type": "object", "properties": {} })
        };
        Some(common::tools::ToolSpec {
          name:        serde_json::json!(runtime_name),
          description: serde_json::json!(tool.description),
          params:      schema,
        })
      })
      .collect()
  }

  async fn provider_adapter(model_info: LlmModel) -> Result<Arc<ProviderAdapter>> {
    if USE_MOCK_PROVIDER {
      return Ok(Arc::new(ProviderAdapter::Mock(Arc::new(MockProvider::new()))));
    }

    let (credentials, provider) = Self::selected_provider(model_info).await?;
    let provider_adapter = Arc::new(build_adapter(&provider.provider(), Some(credentials), provider.base_url)?);

    Ok(provider_adapter)
  }

  pub async fn selected_provider(model_info: LlmModel) -> Result<(BlprntCredentials, ProviderRecord)> {
    let providers_with_credentials = Self::providers_with_credentials().await?;

    let provider = Self::preferred_provider_kind_for_model(&model_info, &providers_with_credentials)
      .ok_or(EngineError::ProviderCredentialsNotFound)?;

    let (mut credentials, provider) =
      providers_with_credentials.get(&provider).cloned().ok_or(EngineError::ProviderCredentialsNotFound)?;

    if provider.is_fnf() {
      credentials =
        Self::maybe_refresh_provider_credentials(provider.provider(), provider.id.clone().key(), credentials).await?;
    }

    Ok((credentials, provider))
  }

  pub(crate) async fn selected_exact_provider(
    provider: Provider,
  ) -> Result<Option<(BlprntCredentials, ProviderRecord)>> {
    let providers = ProviderRepositoryV2::list().await?;
    let Some(provider) = providers.into_iter().find(|candidate| candidate.provider() == provider) else {
      return Ok(None);
    };

    let Ok(mut credentials) = Self::provider_credentials(&provider).await else {
      return Ok(None);
    };

    if provider.is_fnf() {
      let Ok(refreshed_credentials) =
        Self::maybe_refresh_provider_credentials(provider.provider(), provider.id.clone().key(), credentials).await
      else {
        return Ok(None);
      };
      credentials = refreshed_credentials;
    }

    Ok(Some((credentials, provider)))
  }

  pub(crate) async fn visible_enabled_models() -> Result<Vec<LlmModel>> {
    let models = Self::session_resolvable_models().await?;
    let enabled_slugs = Self::enabled_model_slugs_from_store()?;

    Ok(Self::filter_models_by_enabled_slugs(models, &enabled_slugs))
  }

  fn enabled_model_slugs_from_store() -> Result<Vec<String>> {
    let store = Blprnt::handle().store(".models.store").map_err(|e| AppCoreError::FailedToOpenStore(e.to_string()))?;

    match store.get("enabled-model-slugs") {
      None => Ok(Vec::new()),
      Some(slugs) => {
        let slugs = slugs
          .as_array()
          .ok_or_else(|| EngineError::InvalidModelStore("enabled-model-slugs is not an array".into()))?;
        let mut parsed = Vec::with_capacity(slugs.len());
        for slug in slugs {
          let slug = slug
            .as_str()
            .ok_or_else(|| EngineError::InvalidModelStore("enabled-model-slugs contains non-string values".into()))?;
          parsed.push(slug.to_string());
        }
        Ok(parsed)
      }
    }
  }

  fn filter_models_by_enabled_slugs(models: Vec<LlmModel>, enabled_slugs: &[String]) -> Vec<LlmModel> {
    if enabled_slugs.is_empty() {
      return models;
    }

    models.into_iter().filter(|model| enabled_slugs.contains(&model.slug)).collect()
  }

  fn preferred_provider_kind_for_model(
    model_info: &LlmModel,
    providers_with_credentials: &HashMap<Provider, (BlprntCredentials, ProviderRecord)>,
  ) -> Option<Provider> {
    let provider_kinds = providers_with_credentials.keys().copied().collect::<HashSet<_>>();

    if let Some(provider) = Self::strict_provider_kind_for_model(model_info, &provider_kinds) {
      return Some(provider);
    }

    let is_openai_model = model_info.slug.starts_with("openai/");
    let is_anthropic_model = model_info.slug.starts_with("anthropic/");
    let has_openrouter_provider = provider_kinds.contains(&Provider::OpenRouter);

    if (is_openai_model || is_anthropic_model || model_info.slug == "openrouter/auto") && has_openrouter_provider {
      Some(Provider::OpenRouter)
    } else {
      None
    }
  }

  pub(crate) fn strict_provider_kind_for_model(
    model_info: &LlmModel,
    provider_kinds: &HashSet<Provider>,
  ) -> Option<Provider> {
    let model_supports_oauth = model_info.provider_slug.is_some();

    if model_info.slug.starts_with("openai/") {
      if !model_supports_oauth {
        return None;
      }

      if provider_kinds.contains(&Provider::OpenAiFnf) {
        return Some(Provider::OpenAiFnf);
      }

      if provider_kinds.contains(&Provider::OpenAi) {
        return Some(Provider::OpenAi);
      }

      return None;
    }

    if model_info.slug.starts_with("anthropic/") {
      if !model_supports_oauth {
        return None;
      }

      if provider_kinds.contains(&Provider::AnthropicFnf) {
        return Some(Provider::AnthropicFnf);
      }

      if provider_kinds.contains(&Provider::Anthropic) {
        return Some(Provider::Anthropic);
      }

      return None;
    }

    if model_info.slug.starts_with("openrouter/") {
      return provider_kinds.contains(&Provider::OpenRouter).then_some(Provider::OpenRouter);
    }

    if model_info.slug.starts_with("blprnt/") {
      return provider_kinds.contains(&Provider::Blprnt).then_some(Provider::Blprnt);
    }

    if model_info.slug.starts_with("mock/") {
      return provider_kinds.contains(&Provider::Mock).then_some(Provider::Mock);
    }

    None
  }

  pub async fn llm_model(session_model: &SessionRecord) -> Result<LlmModel> {
    let all_enabled_models = Self::session_resolvable_models().await?;

    let model_slug = session_model.model_override().clone();

    let model = all_enabled_models
      .iter()
      .find(|m| m.slug == model_slug)
      .cloned()
      .ok_or(EngineError::ModelNotFound(model_slug))?;

    Ok(model)
  }

  pub(crate) async fn session_resolvable_models() -> Result<Vec<LlmModel>> {
    let providers = ProviderRepositoryV2::list().await?;
    let has_openai = providers.iter().any(|p| p.is_open_ai());
    let has_anthropic = providers.iter().any(|p| p.is_anthropic());

    let store = Blprnt::handle().store("imported-models.json")?;
    let all_models = store.get("models").ok_or(EngineError::ModelNotFound("imported-models.json".into()))?;
    let all_models = serde_json::from_value::<Vec<LlmModel>>(all_models)?;

    Ok(
      all_models
        .iter()
        .filter(|m| {
          m.enabled
            || (has_openai && m.slug.starts_with("openai/"))
            || (has_anthropic && m.slug.starts_with("anthropic/"))
        })
        .cloned()
        .collect(),
    )
  }

  pub(crate) async fn providers_with_credentials() -> Result<HashMap<Provider, (BlprntCredentials, ProviderRecord)>> {
    let providers = ProviderRepositoryV2::list().await?;

    let mut providers_with_credentials = HashMap::new();
    for provider in providers {
      let provider_kind = provider.provider();
      let credentials = Self::provider_credentials(&provider).await?;

      providers_with_credentials.insert(provider_kind, (credentials, provider));
    }

    Ok(providers_with_credentials)
  }

  async fn provider_credentials(provider: &ProviderRecord) -> Result<BlprntCredentials> {
    let provider_uuid =
      Uuid::from_str(&provider.id.key().to_string()).map_err(|e| EngineError::InvalidProviderId(e.to_string()))?;

    match get_stronghold_secret(Vault::Key, provider_uuid).await {
      Some(credentials) => match (looks_like_json(&credentials), serde_json::from_str(&credentials)) {
        (false, _) => Ok(BlprntCredentials::ApiKey(credentials)),
        (true, Ok(credentials)) => Ok(credentials),
        (true, Err(e)) => Err(EngineError::FailedToDeserializeCredentials(e.to_string()).into()),
      },
      None => Err(EngineError::ProviderCredentialsNotFound.into()),
    }
  }

  async fn maybe_refresh_provider_credentials(
    provider: Provider,
    provider_uuid: Uuid,
    credentials: BlprntCredentials,
  ) -> Result<BlprntCredentials> {
    match credentials {
      BlprntCredentials::OauthToken(token) => match (provider, token) {
        (Provider::OpenAi | Provider::OpenAiFnf, OauthToken::OpenAi(mut t)) => {
          let now = now_ms();
          let effective_expires_at_ms = t.expires_at_ms.min(now.saturating_add(OPENAI_OAUTH_MAX_TTL_MS));
          if !should_refresh_oauth_token(now, effective_expires_at_ms, OAUTH_REFRESH_SKEW_MS) {
            return Ok(BlprntCredentials::OauthToken(OauthToken::OpenAi(t)));
          }
          if t.refresh_token.is_empty() {
            return Err(EngineError::ProviderCredentialsNotFound.into());
          }

          let refreshed = OpenAiOauth::refresh_with_refresh_token(
            oauth_consts::openai::TOKEN_URL,
            oauth_consts::openai::CLIENT_ID,
            &t.refresh_token,
          )
          .await?;

          if let Some(refreshed) = refreshed {
            t.access_token = refreshed.access_token;
            t.refresh_token = refreshed.refresh_token;
            t.expires_at_ms = now_ms().saturating_add(OPENAI_OAUTH_MAX_TTL_MS);

            let updated = BlprntCredentials::OauthToken(OauthToken::OpenAi(t));
            set_stronghold_secret(Vault::Key, provider_uuid, &serde_json::to_string(&updated)?).await?;
            return Ok(updated);
          }

          Ok(BlprntCredentials::OauthToken(OauthToken::OpenAi(t)))
        }

        (Provider::Anthropic | Provider::AnthropicFnf, OauthToken::Anthropic(mut t)) => {
          if !should_refresh_oauth_token(now_ms(), t.expires_at_ms, OAUTH_REFRESH_SKEW_MS) {
            return Ok(BlprntCredentials::OauthToken(OauthToken::Anthropic(t)));
          }
          if t.refresh_token.is_empty() {
            return Err(EngineError::ProviderCredentialsNotFound.into());
          }

          let refreshed = AnthropicOauth::refresh_with_refresh_token(&t.refresh_token).await?;
          if let Some(refreshed) = refreshed {
            t.access_token = refreshed.access_token;
            if !refreshed.refresh_token.is_empty() {
              t.refresh_token = refreshed.refresh_token;
            }
            t.expires_at_ms = now_ms().saturating_add(refreshed.expires_in.saturating_mul(1000));

            let updated = BlprntCredentials::OauthToken(OauthToken::Anthropic(t));
            set_stronghold_secret(Vault::Key, provider_uuid, &serde_json::to_string(&updated)?).await?;
            return Ok(updated);
          }

          Ok(BlprntCredentials::OauthToken(OauthToken::Anthropic(t)))
        }

        (_, token) => Ok(BlprntCredentials::OauthToken(token)),
      },
      other => Ok(other),
    }
  }
}

fn looks_like_json(s: &str) -> bool {
  let trimmed = s.trim();
  (trimmed.starts_with('{') && trimmed.ends_with('}')) || (trimmed.starts_with('[') && trimmed.ends_with(']'))
}

fn now_ms() -> u64 {
  SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or(Duration::ZERO).as_millis() as u64
}

fn should_refresh_oauth_token(now_ms: u64, expires_at_ms: u64, skew_ms: u64) -> bool {
  now_ms.saturating_add(skew_ms) >= expires_at_ms
}

#[cfg(test)]
mod tests {
  use common::agent::AgentKind;
  use common::shared::prelude::LlmModel;
  use common::shared::prelude::McpToolDescriptor;

  use super::RuntimeContext;

  #[test]
  fn mcp_tool_specs_include_runtime_name_for_main_and_subagent() {
    let tools = vec![McpToolDescriptor {
      server_id:    "server-a".to_string(),
      name:         "tool-a".to_string(),
      description:  "tool description".to_string(),
      input_schema: serde_json::json!({"type":"object","properties":{"query":{"type":"string"}}}),
    }];

    let main_specs = RuntimeContext::mcp_tool_specs(tools.clone(), AgentKind::Planner, false);
    let subagent_specs = RuntimeContext::mcp_tool_specs(tools, AgentKind::Planner, true);

    assert_eq!(main_specs.len(), 1);
    assert_eq!(subagent_specs.len(), 1);
    assert_eq!(main_specs[0].name, serde_json::json!("mcp__server-a__tool-a"));
    assert_eq!(subagent_specs[0].name, serde_json::json!("mcp__server-a__tool-a"));
  }

  #[test]
  fn mcp_tool_specs_normalize_non_object_schema() {
    let tools = vec![McpToolDescriptor {
      server_id:    "server-b".to_string(),
      name:         "tool-b".to_string(),
      description:  "tool description".to_string(),
      input_schema: serde_json::json!("invalid"),
    }];

    let specs = RuntimeContext::mcp_tool_specs(tools, AgentKind::Crew, false);
    assert_eq!(specs.len(), 1);
    assert_eq!(
      specs[0].params,
      serde_json::json!({
        "type": "object",
        "properties": {}
      })
    );
  }

  #[test]
  fn looks_like_json_detects_wrapped_payloads_only() {
    assert!(super::looks_like_json("{\"a\":1}"));
    assert!(super::looks_like_json("[1,2,3]"));
    assert!(!super::looks_like_json("plain-text"));
  }

  #[test]
  fn strict_provider_kind_for_model_prefers_same_family_fnf_without_fallback() {
    use std::collections::HashSet;

    use common::shared::prelude::Provider;

    let model =
      LlmModel { slug: "openai/gpt-5.1".to_string(), provider_slug: Some("openai".to_string()), ..Default::default() };

    let providers = HashSet::from([Provider::OpenAiFnf, Provider::OpenRouter]);

    assert_eq!(RuntimeContext::strict_provider_kind_for_model(&model, &providers), Some(Provider::OpenAiFnf));

    let providers = HashSet::from([Provider::OpenRouter]);

    assert_eq!(RuntimeContext::strict_provider_kind_for_model(&model, &providers), None);
  }

  #[test]
  fn strict_provider_kind_for_model_keeps_exact_available_variant() {
    use std::collections::HashSet;

    use common::shared::prelude::Provider;

    let model = LlmModel {
      slug: "anthropic/claude-sonnet".to_string(),
      provider_slug: Some("anthropic".to_string()),
      ..Default::default()
    };

    let providers = HashSet::from([Provider::Anthropic]);
    assert_eq!(RuntimeContext::strict_provider_kind_for_model(&model, &providers), Some(Provider::Anthropic));

    let providers = HashSet::from([Provider::AnthropicFnf]);
    assert_eq!(RuntimeContext::strict_provider_kind_for_model(&model, &providers), Some(Provider::AnthropicFnf));
  }
}
