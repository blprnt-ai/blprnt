import { invoke } from '@tauri-apps/api/core'
import { openUrl } from '@tauri-apps/plugin-opener'
import { load, type Store } from '@tauri-apps/plugin-store'
import { CircleDot } from 'lucide-react'
import { useCallback, useEffect, useState } from 'react'
import { InfoBox } from '@/components/atoms/boxes'
import { Button } from '@/components/atoms/button'
import { Switch } from '@/components/atoms/switch'
import { basicToast } from '@/components/atoms/toaster'
import { Section } from '@/components/organisms/page/section'
import { SectionField } from '@/components/organisms/page/section-field'
import { EventType, globalEventBus } from '@/lib/events'

const slackStoreFile = 'blprnt.json'
const slackEnabledKey = 'slack_enabled'

type SlackStatus = {
  enabled: boolean
  connected: boolean
  last_error: string | null
}

export const IntegrationsPage = () => {
  return <SlackSection />
}

const SlackSection = () => {
  const [store, setStore] = useState<Store | null>(null)
  const [isLoaded, setIsLoaded] = useState(false)
  const [isEnabled, setIsEnabled] = useState(false)
  const [status, setStatus] = useState<SlackStatus | null>(null)
  const [isConnecting, setIsConnecting] = useState(false)

  const refreshStatus = useCallback(async () => {
    try {
      const result = await invoke<SlackStatus>('slack_status')
      setStatus(result)
      setIsEnabled(result.enabled)
    } catch {
      setStatus(null)
    }
  }, [])

  useEffect(() => {
    const loadStore = async () => {
      const store = await load(slackStoreFile)
      const enabled = (await store.get<boolean>(slackEnabledKey)) ?? false

      setStore(store)
      setIsEnabled(enabled)
      setIsLoaded(true)
    }

    const unsub = globalEventBus.subscribe(
      EventType.TunnelMessage,
      () => refreshStatus(),
      (event) => event.payload.type === 'slack_oauth_callback',
    )
    void loadStore()

    return () => unsub()
  }, [refreshStatus])

  useEffect(() => {
    void refreshStatus()
  }, [refreshStatus])

  useEffect(() => {
    if (!store || !isLoaded) return
    store.set(slackEnabledKey, isEnabled)
    void invoke('slack_set_enabled', { enabled: isEnabled })
  }, [isEnabled, isLoaded, store])

  const handleToggleEnabled = (checked: boolean) => {
    setIsEnabled(checked)
  }

  const handleConnect = async () => {
    setIsConnecting(true)
    try {
      const result = await invoke<{ url: string }>('slack_start_oauth')
      await openUrl(result.url)
      await refreshStatus()
    } catch (error) {
      basicToast.error({ description: (error as Error).message, title: 'Slack connection failed' })
    } finally {
      setIsConnecting(false)
    }
  }

  const handleDisconnect = async () => {
    setIsEnabled(false)
    try {
      await invoke('slack_disconnect')
      await refreshStatus()
    } catch {
      setStatus((prev) => (prev ? { ...prev, connected: false } : prev))
    }
  }

  return (
    <Section>
      <SectionField
        title={
          <div className="flex flex-col gap-2">
            <div>Slack</div>
            <div className="text-muted-foreground text-sm font-light">
              Connect Slack for DM-only notifications and replies.
            </div>
            {status?.connected && (
              <div className="flex items-center gap-1 text-green-500 text-xs font-light -mt-1">
                <CircleDot className="size-2.5 animate-pulse -mt-0.5" /> Connected
              </div>
            )}
          </div>
        }
      >
        <div className="w-full space-y-4">
          {status?.last_error && (
            <div className="flex items-center gap-4">
              <div className="flex flex-col gap-1">
                <span className="text-xs text-destructive">{status.last_error}</span>
              </div>
            </div>
          )}

          <div className="flex items-center gap-4">
            <div className="flex flex-col gap-1">
              <span>Enable</span>
              {!status?.connected && (
                <span className="text-muted-foreground text-xs">Requires connected Slack account</span>
              )}
            </div>
            <Switch checked={isEnabled} disabled={!status?.connected} onCheckedChange={handleToggleEnabled} />
          </div>

          <div className="flex flex-wrap gap-2">
            {!status?.connected ? (
              <Button disabled={isConnecting} variant="outline" onClick={handleConnect}>
                {isConnecting ? 'Connecting...' : 'Connect Slack'}
              </Button>
            ) : (
              <Button variant="secondary" onClick={handleDisconnect}>
                Disconnect
              </Button>
            )}
          </div>
        </div>

        <InfoBox className="w-full">
          <div className="flex flex-col gap-1 mt-2">
            <div>Slack integration is in alpha testing and may not work as expected.</div>
          </div>
        </InfoBox>
      </SectionField>
    </Section>
  )
}
