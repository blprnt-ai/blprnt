export type McpOauthMetadataDto = {
  authorization_endpoint: string
  issuer?: string | null
  registration_endpoint?: string | null
  scopes_supported: string[]
  token_endpoint: string
}
