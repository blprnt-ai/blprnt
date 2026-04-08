import type { McpServerAuthState } from './McpServerAuthState'

export type McpServerPatchPayload = {
  display_name?: string | null
  description?: string | null
  transport?: string | null
  endpoint_url?: string | null
  enabled?: boolean | null
  auth_state?: McpServerAuthState | null
  auth_summary?: string | null
}
