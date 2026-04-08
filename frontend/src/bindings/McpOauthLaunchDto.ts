import type { McpOauthMetadataDto } from './McpOauthMetadataDto'
import type { McpServerAuthState } from './McpServerAuthState'

export type McpOauthLaunchDto = {
  server_id: string
  authorization_url: string
  redirect_uri: string
  auth_state: McpServerAuthState
  auth_summary?: string | null
  metadata?: McpOauthMetadataDto | null
  suggested_scopes: string[]
}
