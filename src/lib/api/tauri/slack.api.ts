import { invoke } from '@tauri-apps/api/core'

export type SlackStartOAuthResponse = {
  url: string
  state: string
}

export type SlackStatus = {
  enabled: boolean
  connected: boolean
  last_error: string | null
}

export const slackStartOauth = () => invoke<SlackStartOAuthResponse>('slack_start_oauth')

export const slackStatus = () => invoke<SlackStatus>('slack_status')

export const slackSetEnabled = (enabled: boolean) => invoke('slack_set_enabled', { enabled })

export const slackDisconnect = () => invoke('slack_disconnect')
