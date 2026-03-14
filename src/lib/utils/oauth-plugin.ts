import { invoke } from '@tauri-apps/api/core'
import { type Event, once } from '@tauri-apps/api/event'

/**
 * Configuration options for the OAuth server.
 */
export interface OauthConfig {
  /**
   * An array of port numbers the server should try to bind to.
   * If not provided, the server will use a random available port.
   */
  ports?: number[]

  /**
   * Custom HTML response sent to the user after being redirected.
   * If not provided, a default response will be used.
   */
  response?: string
}

export async function start(config?: OauthConfig): Promise<number> {
  return await invoke<number>('plugin:oauth|start', { config })
}

export async function cancel(port: number): Promise<void> {
  await invoke<void>('plugin:oauth|cancel', { port })
}

interface TokenResponse {
  token: string
  user_id: string
}

export function onToken(callback: (token: TokenResponse) => void): Promise<() => void> {
  return once('oauth://token', (event: Event<TokenResponse>) => {
    callback(event.payload)
  })
}

export function onJwt(callback: (jwt: string) => void): Promise<() => void> {
  return once('oauth://jwt', (event: Event<string>) => {
    callback(event.payload)
  })
}
