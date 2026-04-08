import type { McpServerAuthState } from './McpServerAuthState'

export type McpServerDto = {
  id: string
  display_name: string
  description: string
  transport: string
  endpoint_url: string
  auth_state: McpServerAuthState
  auth_summary?: string | null
  enabled: boolean
  created_at: string
  updated_at: string
}
