// biome-ignore assist/source/useSortedKeys: I want specific ordering
export const ERROR_MESSAGES: Record<string, { title: string; description: string }> = {
  // Provider errors
  user_cancelled: {
    description: 'You cancelled the request.',
    title: 'Request Cancelled',
  },
  external_network: {
    description: 'Could not connect to the AI provider. Check your internet connection and try again.',
    title: 'Network Issue',
  },
  decoding_failed: {
    description: 'Failed to decode the AI response. This is usually a temporary issue.',
    title: 'Response Error',
  },
  llm_mistake: {
    description: 'The AI made an error in its response. Trying again may help.',
    title: 'AI Response Issue',
  },
  llm_error: {
    description: 'The AI provider returned an error. This is usually temporary.',
    title: 'AI Error',
  },
  llm_unknown_error: {
    description: 'An unexpected error occurred with the AI provider.',
    title: 'Unknown AI Error',
  },
  rate_limit: {
    description: "You've hit the rate limit. Please wait a moment before trying again or upgrade your plan.",
    title: 'Rate Limited',
  },
  rate_limit_upstream: {
    description: 'The AI provider has temporarily rate-limited all requests. Please retry shortly.',
    title: 'Rate Limited',
  },
  bad_request: {
    description: 'The request was invalid. This may be a configuration issue.',
    title: 'Invalid Request',
  },
  unauthorized: {
    description:
      'Your API key is invalid or expired. Please check your plan. If you need additional help, please contact support.',
    title: 'Not Authorized',
  },
  cannot_clone_request: {
    description:
      'An internal error occurred while preparing the request. Please try again. If the problem persists, please contact support.',
    title: 'Request Error',
  },
  utf8_error: {
    description: 'Failed to process text encoding. This is usually a temporary issue.',
    title: 'Encoding Error',
  },
  parser_error: {
    description: 'Failed to parse the response from the AI provider. This is usually a temporary issue.',
    title: 'Parse Error',
  },
  transport_error: {
    description:
      'Failed to communicate with the AI provider. Check your connection. If the problem persists, please contact support.',
    title: 'Connection Error',
  },
  middleware_transport: {
    description:
      'Network middleware error. Check your connection and try again. If the problem persists, please contact support.',
    title: 'Connection Error',
  },
  invalid_content_type: {
    description: 'Received an unexpected response format from the AI provider. This is usually a temporary issue.',
    title: 'Invalid Response',
  },
  invalid_status_code: {
    description:
      'The AI provider returned an unexpected status. Try again. If the problem persists, please contact support.',
    title: 'Server Error',
  },
  stream_ended: {
    description: 'The response stream ended unexpectedly. This is usually a temporary issue.',
    title: 'Stream Ended',
  },
  decode_error: {
    description: 'Failed to decode the response data.',
    title: 'Decode Error',
  },
  auth_headers: {
    description:
      'Failed to set authentication headers, check your plan. If you need additional help, please contact support.',
    title: 'Authentication Error',
  },
  timeout: {
    description: 'The request took too long. The AI provider may be experiencing high load.',
    title: 'Request Timeout',
  },
  canceled: {
    description: 'The request was cancelled.',
    title: 'Cancelled',
  },
  not_supported: {
    description:
      'This feature is not supported by the current AI provider. Please contact support if you need additional help.',
    title: 'Not Supported',
  },
  not_supported_streaming: {
    description: 'Streaming is not supported for this model or provider. Please contact support.',
    title: 'Streaming Not Supported',
  },
  upstream_error: {
    description:
      'The AI provider encountered an internal error. Try again. If the problem persists, please contact support.',
    title: 'Upstream Error',
  },
  internal: {
    description: 'An internal error occurred. Please try again. If the problem persists, please contact support.',
    title: 'Internal Error',
  },
  invalid_provider: {
    description: 'The selected AI provider is not valid or configured. Please contact support.',
    title: 'Invalid Provider',
  },
  encoding_error: {
    description: 'Failed to encode the request data. This is usually a temporary issue.',
    title: 'Encoding Error',
  },
  invalid_schema: {
    description: 'The response schema is invalid. This is usually a temporary issue.',
    title: 'Schema Error',
  },
  content_moderation: {
    description: 'Your request was blocked by content moderation policies.',
    title: 'Content Blocked',
  },
  insufficient_credits: {
    description: 'You have hit your monthly credit limit. Please upgrade your plan to continue.',
    title: 'Insufficient Credits',
  },
  model_unavailable: {
    description: 'The selected model is temporarily unavailable. Try a different model or wait.',
    title: 'Model Unavailable',
  },
  provider_unavailable: {
    description: 'The AI provider is temporarily unavailable. Please try again later.',
    title: 'Provider Unavailable',
  },
  gateway_timeout: {
    description: 'The AI timed out. Please try again.',
    title: 'Gateway Timeout',
  },
  server_error: {
    description: 'The AI experienced a server error. Please try again.',
    title: 'Server Error',
  },
  invalid_api_key: {
    description:
      'Your API key is invalid. Please check your plan. If you need additional help, please contact support.',
    title: 'Invalid API Key',
  },
  context_length_exceeded: {
    description:
      'The conversation is too long for this model. Try starting a new session or using a model with a larger context window. Alternatively, you can try reducing the context window removing some of the previous messages.',
    title: 'Context Too Long',
  },

  // Engine errors
  context_window_too_large: {
    description:
      'The conversation context exceeds the model limit. Upgrade your plan to continue. Alternatively, you can try reducing the context window removing some of the previous messages.',
    title: 'Context Too Large',
  },
  failed_to_parse_tool_args: {
    description: 'Tool call failure. The AI will retry the request.',
    title: 'Tool Argument Error',
  },
  provider_channel_error: {
    description: 'Communication error with the AI.',
    title: 'Provider Error',
  },
  provider_not_found: {
    description: 'The AI was not found. Check your configuration.',
    title: 'Provider Not Found',
  },
  failed_to_deserialize_credentials: {
    description: 'Failed to load your credentials. Try logging out and signing in again.',
    title: 'Credentials Error',
  },
  provider_credentials_not_found: {
    description: 'No credentials found for this AI. Try logging out and signing in again.',
    title: 'Missing Credentials',
  },
  no_auto_router_models: {
    description:
      'No auto suitable models found for auto. Either select a model override for this session or enable and auto-router compatible model in your blprnt settings.',
    title: 'No Auto Router Models',
  },
  invalid_model_store: {
    description: 'Model settings are corrupted. Try resetting your model settings in preferences.',
    title: 'Model Settings Error',
  },
  model_not_found: {
    description: 'The selected model is not available for your plan. Please choose a different model.',
    title: 'Model Not Found',
  },
  invalid_provider_id: {
    description: 'The AI provider configuration is invalid. Try reselecting your provider.',
    title: 'Provider Configuration Error',
  },
  invalid_model_slug: {
    description: 'The selected model identifier is invalid. Please choose a different model.',
    title: 'Model Configuration Error',
  },
  session_project_missing: {
    description: 'This session no longer has an associated project. Update the session settings and try again.',
    title: 'Missing Project',
  },
  session_personality_missing: {
    description: 'This session is missing a personality. Update the session settings and try again.',
    title: 'Missing Personality',
  },
  invalid_subagent_id: {
    description: 'The AI tried to call a subagent with an invalid ID.',
    title: 'Subagent Error',
  },
  failed_to_create_session: {
    description: 'Failed to create the session. Please try again.',
    title: 'Session Error',
  },
  failed_to_open_session: {
    description: 'Failed to open the session. Please try again.',
    title: 'Session Error',
  },
  plan_already_attached_to_different_session: {
    description:
      'The plan is already attached to a different session. Please detach the plan from the other session and try again.',
    title: 'Plan Error',
  },

  // Auth errors
  failed_to_insert_bearer: {
    description: 'Failed to set authentication token. Try logging out and signing in again.',
    title: 'Auth Error',
  },
  failed_to_insert_api_key: {
    description: 'Failed to save API key. Try again, if the problem persists, please contact support.',
    title: 'Auth Error',
  },
  invalid_base_authorize_url: {
    description: 'Invalid authorization URL.',
    title: 'Auth Error',
  },
  failed_to_bind_fixed_port: {
    description: 'Failed to bind to port for authentication..',
    title: 'Auth Error',
  },
  failed_to_bind_local_callback_listener: {
    description: 'Failed to set up authentication listener.',
    title: 'Auth Error',
  },
  failed_to_open_browser: {
    description: 'Failed to open your browser for sign in.',
    title: 'Browser Error',
  },
  failed_to_get_local_address: {
    description: 'Failed to get local network address.',
    title: 'Auth Error',
  },
  missing_authorization_code: {
    description: 'Authorization code was not received. Try logging out and signing in again.',
    title: 'Auth Error',
  },
  state_mismatch: {
    description: 'Security state mismatch. Try logging out and signing in again.',
    title: 'Auth Security Error',
  },
  missing_state: {
    description: 'Authentication state was missing. Try logging out and signing in again.',
    title: 'Auth Error',
  },
  failed_to_exchange_code_for_token: {
    description: 'Failed to complete authentication. Try logging out and signing in again.',
    title: 'Auth Error',
  },
  failed_to_parse_token_response: {
    description: 'Failed to process authentication response. Try logging out and signing in again.',
    title: 'Auth Error',
  },
  failed_to_send_token_request: {
    description:
      'Failed to send authentication request. Check your connection. If the problem persists, please contact support.',
    title: 'Auth Error',
  },
  failed_to_refresh_with_refresh_token: {
    description:
      'Your provider session has expired. Please sign in to codex/claude to refresh your session then re-link in settings -> providers.',
    title: 'Session Expired',
  },
  failed_to_parse_refresh_response: {
    description: 'Failed to refresh your session. Try logging out and signing in again.',
    title: 'Auth Error',
  },
  failed_to_send_refresh_request: {
    description:
      'Failed to refresh your session. Check your connection. If the problem persists, please contact support.',
    title: 'Auth Error',
  },
  failed_to_parse_id_token: {
    description: 'Failed to parse identity token. Try logging out and signing in again.',
    title: 'Auth Error',
  },
  failed_to_decode_id_token: {
    description: 'Failed to decode identity token. Try logging out and signing in again.',
    title: 'Auth Error',
  },
  failed_to_get_oauth_token: {
    description: 'Failed to get authentication token. Try logging out and signing in again.',
    title: 'Auth Error',
  },
  failed_to_start_oauth: {
    description: 'Failed to start authentication process. Try logging out and signing in again.',
    title: 'Auth Error',
  },
  failed_to_sign_in: {
    description: 'Failed to sign in. Try logging out and signing in again.',
    title: 'Sign In Failed',
  },
  failed_to_get_subscription: {
    description: 'Failed to retrieve your subscription information. Try logging out and signing in again.',
    title: 'Subscription Error',
  },

  // AppCore errors
  failed_to_open_store: {
    description: 'Failed to open the data store. Try restarting the app.',
    title: 'Storage Error',
  },
  project_not_found: {
    description: 'The project could not be found. This should not happen, please contact support.',
    title: 'Project Not Found',
  },
  session_not_found: {
    description: 'The session could not be found. This should not happen, please contact support.',
    title: 'Session Not Found',
  },
  indexing_not_enabled: {
    description: 'Indexing is not enabled for this project. Please contact support.',
    title: 'Indexing Not Enabled',
  },
  codex_credentials_not_found: {
    description: 'Codex credentials not found. Try logging out and signing in again.',
    title: 'Codex Credentials Not Found',
  },
  claude_credentials_not_found: {
    description: 'Claude credentials not found. Try logging out and signing in again.',
    title: 'Claude Credentials Not Found',
  },

  // API errors
  failed_to_get_user: {
    description: 'Failed to retrieve your user information. Try logging out and signing in again.',
    title: 'User Error',
  },
  failed_to_initialize_user: {
    description: 'Failed to initialize your user account. Try logging out and signing in again.',
    title: 'Initialization Error',
  },
  failed_to_get_models: {
    description: 'Failed to retrieve available models. Try logging out and signing in again.',
    title: 'Models Error',
  },
  failed_to_get_blprnt_response: {
    description: 'Failed to get a response. Try logging out and signing in again.',
    title: 'Blprnt Error',
  },
}

export const CATEGORY_LABELS: Record<string, string> = {
  auth: 'Authentication',
  engine: 'Engine',
  internal: 'System',
  network: 'Network',
  provider: 'AI Provider',
  tool: 'Tool',
}
