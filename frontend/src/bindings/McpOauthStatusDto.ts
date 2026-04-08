import type { McpOauthMetadataDto } from './McpOauthMetadataDto'
import type { McpServerAuthState } from './McpServerAuthState'

export type McpOauthStatusDto = {
  server_id: string
  auth_state: McpServerAuthState
  auth_summary?: string | null
  authorization_url?: string | null
  has_token: boolean
  metadata?: McpOauthMetadataDto | null
  scopes: string[]
  token_expires_at?: number | null
}
