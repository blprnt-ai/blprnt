use common::shared::prelude::Provider;

pub(crate) fn base_model_for_provider(provider: Provider) -> &'static str {
  match provider {
    Provider::Anthropic | Provider::AnthropicFnf => "claude-haiku-4-5",
    Provider::OpenAi | Provider::OpenAiFnf => "gpt-5.1-codex-mini",
    Provider::OpenRouter => "liquid/lfm2-8b-a1b",
    Provider::Blprnt => "LiquidAI/LFM2.5-1.2B-Instruct-GGUF",
    Provider::Mock => "mock-reasoning-classifier",
  }
}
