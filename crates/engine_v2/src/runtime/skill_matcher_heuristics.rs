use std::collections::HashMap;
use std::sync::Arc;

use common::skills_utils::SkillsUtils;
use common::tools::SkillItem;
use persistence::prelude::MessageRecord;
use providers::ProviderAdapter;
use serde::Deserialize;
use serde_json::json;
use tokio_util::sync::CancellationToken;

use crate::runtime::provider_model_heuristics::base_model_for_provider;

const SKILL_MATCHER_SYSTEM_PROMPT: &str = r#"You are a strict skill matcher.
You will be given recent conversation/user messages plus available skills metadata. Determine what the user is trying to accomplish and select skills that would help complete that work.
Match to context and intent, not keyword overlap; consider implicit context and constraints from earlier messages.
Output must be only valid JSON. No prose, no markdown, no code fences.
Return exactly one JSON object with this exact schema and key: {\"skills\":[\"...\"]}.
Rules:
- Select 0 to 3 skills maximum.
- If there are meta instruction in the prompt, you should apply the skill mentioned in the meta instruction.
- Use only exact skill id values from available_skills. Never use display names.
- Prioritize precision over recall; avoid false positives.
- If the request is ambiguous or uncertain, return {\"skills\":[]}.
- Deduplicate skills and preserve relevance order.
- No extra keys or text.
- Do not wrap the JSON in code fences.
- DO NOT WRAP THE JSON IN CODE FENCES.
Example valid output: {\"skills\":[\"rust-debugging\",\"tauri-ipc\"]}."#;

const MAX_SELECTED_SKILLS: usize = 3;

#[derive(Deserialize)]
struct SkillMatcherResponse {
  skills: Vec<String>,
}

pub async fn match_skills_for_turn(
  current_prompt: String,
  recent_user_messages: Vec<MessageRecord>,
  provider_adapter: Arc<ProviderAdapter>,
  cancel_token: CancellationToken,
) -> Vec<String> {
  let Ok(available_skills) = SkillsUtils::list_skills() else {
    tracing::warn!("Skill matcher: failed to list skills");
    return vec![];
  };

  if available_skills.is_empty() {
    return vec![];
  }

  let user_messages = collect_user_messages_for_matching(current_prompt, recent_user_messages);
  let user_prompt = build_skill_matcher_prompt(&user_messages, &available_skills);
  let model = base_model_for_provider(provider_adapter.provider()).to_string();

  let response = provider_adapter
    .one_off_request(user_prompt, SKILL_MATCHER_SYSTEM_PROMPT.to_string(), Some(model), cancel_token)
    .await;

  match response {
    Ok(chat_basic) => {
      let raw = chat_basic.messages.first().cloned().unwrap_or_default();
      parse_and_validate_skill_ids(&raw, &available_skills)
    }
    Err(error) => {
      tracing::warn!("Skill matcher failed: {}", error);
      vec![]
    }
  }
}

fn collect_user_messages_for_matching(current_prompt: String, recent_user_messages: Vec<MessageRecord>) -> Vec<String> {
  let mut collected = vec![current_prompt];
  let mut counting_messages = 0usize;

  for message in recent_user_messages {
    if counting_messages >= 5 {
      break;
    }

    let text = message_text_for_matcher(&message);
    if text.is_empty() {
      continue;
    }

    if word_count(&text) >= 10 {
      counting_messages += 1;
    }

    collected.push(text);
  }

  collected
}

fn message_text_for_matcher(message: &MessageRecord) -> String {
  match message.content() {
    common::shared::prelude::MessageContent::Text(text) => text.text.trim().to_string(),
    _ => String::new(),
  }
}

fn word_count(text: &str) -> usize {
  text.split_whitespace().count()
}

fn build_skill_matcher_prompt(user_messages: &[String], available_skills: &[SkillItem]) -> String {
  let payload = json!({
    "user_messages": user_messages,
    "available_skills": available_skills,
    "output_schema": { "skills": ["skill_id"] },
    "constraints": {
      "max_skills": MAX_SELECTED_SKILLS,
      "ids_only": true
    }
  });

  payload.to_string()
}

fn parse_and_validate_skill_ids(raw: &str, available_skills: &[SkillItem]) -> Vec<String> {
  let raw = raw.strip_prefix("```json").unwrap_or(raw);
  let raw = raw.strip_suffix("```").unwrap_or(raw);

  let parsed = serde_json::from_str::<SkillMatcherResponse>(raw);
  let Ok(parsed) = parsed else {
    tracing::warn!("Skill matcher returned invalid JSON: {}", raw);
    return vec![];
  };

  let mut normalized_to_id = HashMap::new();
  for skill in available_skills {
    normalized_to_id.insert(normalize_skill_key(&skill.id), skill.id.clone());
    normalized_to_id.insert(normalize_skill_key(&skill.name), skill.id.clone());
  }

  let mut result = Vec::new();
  for candidate in parsed.skills {
    if result.len() >= MAX_SELECTED_SKILLS {
      break;
    }

    let normalized = normalize_skill_key(&candidate);
    let Some(skill_id) = normalized_to_id.get(&normalized) else {
      continue;
    };

    if result.contains(skill_id) {
      continue;
    }

    result.push(skill_id.clone());
  }

  result
}

fn normalize_skill_key(value: &str) -> String {
  value.trim().to_lowercase().replace('_', "-")
}
