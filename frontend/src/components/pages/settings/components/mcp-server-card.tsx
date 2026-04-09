import { CableIcon, ExternalLinkIcon, RefreshCwIcon } from 'lucide-react'
import { observer } from 'mobx-react-lite'
import type { McpServerDto } from '@/bindings/McpServerDto'
import { IssueBadge } from '@/components/pages/issue/components/issue-badge'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { statusBadge, statusBadgeDefault } from '@/lib/status-colors'
import { cn } from '@/lib/utils'
import { useMcpSettingsViewmodel } from '../mcp-settings.viewmodel'

interface McpServerCardProps {
  server: McpServerDto
}

export const McpServerCard = observer(({ server }: McpServerCardProps) => {
  const viewmodel = useMcpSettingsViewmodel()
  const oauthStatus = viewmodel.getOauthStatus(server.id)
  const launch = viewmodel.getLaunch(server.id)
  const draft = viewmodel.getCompletionDraft(server.id)
  const isRefreshing = viewmodel.isRefreshingOauthId === server.id

  return (
    <Card className="border-border/70 py-0">
      <CardHeader className="border-b py-4">
        <div className="flex items-start justify-between gap-3">
          <div className="space-y-2">
            <CardTitle>{server.display_name}</CardTitle>
            <p className="text-sm text-muted-foreground">{server.description}</p>
          </div>
          <IssueBadge className={cn(statusBadge[server.auth_state] ?? statusBadgeDefault)}>
            {formatAuthState(server.auth_state)}
          </IssueBadge>
        </div>
      </CardHeader>

      <CardContent className="flex flex-col gap-4 py-4">
        <div className="flex flex-wrap gap-2">
          <IssueBadge>{server.transport}</IssueBadge>
          <IssueBadge>{server.enabled ? 'Enabled' : 'Disabled'}</IssueBadge>
          {oauthStatus?.has_token ? <IssueBadge>Token stored</IssueBadge> : null}
        </div>

        <div className="grid gap-3 md:grid-cols-2">
          <InfoBlock label="Endpoint" value={server.endpoint_url} />
          <InfoBlock
            label="Auth summary"
            value={oauthStatus?.auth_summary ?? server.auth_summary ?? 'No summary yet.'}
          />
        </div>

        {oauthStatus ? (
          <div className="grid gap-3 md:grid-cols-2">
            <InfoBlock
              label="Scopes"
              value={oauthStatus.scopes.length > 0 ? oauthStatus.scopes.join(', ') : 'No scopes recorded.'}
            />
            <InfoBlock label="Token expiry" value={formatExpiry(oauthStatus.token_expires_at)} />
          </div>
        ) : null}

        {launch?.redirect_uri ? <InfoBlock label="Redirect URI" value={launch.redirect_uri} /> : null}

        <div className="flex flex-wrap gap-2">
          <Button size="sm" type="button" variant="outline" onClick={() => viewmodel.openEdit(server)}>
            Edit
          </Button>
          <Button
            disabled={isRefreshing}
            size="sm"
            type="button"
            variant="outline"
            onClick={() => void viewmodel.refreshOauth(server.id)}
          >
            <RefreshCwIcon className={cn('size-4', isRefreshing && 'animate-spin')} />
            Refresh status
          </Button>
          {server.auth_state === 'reconnect_required' ? (
            <Button
              disabled={isRefreshing}
              size="sm"
              type="button"
              onClick={() => void viewmodel.reconnectOauth(server)}
            >
              <CableIcon className="size-4" />
              Reconnect
            </Button>
          ) : (
            <Button disabled={isRefreshing} size="sm" type="button" onClick={() => void viewmodel.launchOauth(server)}>
              <ExternalLinkIcon className="size-4" />
              {server.auth_state === 'connected' ? 'Launch auth' : 'Connect'}
            </Button>
          )}
        </div>

        <div className="grid gap-3 rounded-sm border border-border/70 p-3 md:grid-cols-2">
          <div className="flex flex-col gap-2">
            <Label htmlFor={`mcp-code-${server.id}`}>OAuth code</Label>
            <Input
              id={`mcp-code-${server.id}`}
              value={draft.code}
              onChange={(event) => viewmodel.setCompletionCode(server.id, event.target.value)}
            />
          </div>
          <div className="flex flex-col gap-2">
            <Label htmlFor={`mcp-state-${server.id}`}>OAuth state</Label>
            <Input
              id={`mcp-state-${server.id}`}
              value={draft.state}
              onChange={(event) => viewmodel.setCompletionState(server.id, event.target.value)}
            />
          </div>
          <div className="md:col-span-2 flex items-center justify-between gap-3">
            <p className="text-xs text-muted-foreground">
              Use this only if the provider gives you a code/state pair to paste back manually.
            </p>
            <Button
              disabled={isRefreshing || !draft.code.trim() || !draft.state.trim()}
              size="sm"
              type="button"
              onClick={() => void viewmodel.completeOauth(server.id)}
            >
              Complete OAuth
            </Button>
          </div>
        </div>
      </CardContent>
    </Card>
  )
})

const InfoBlock = ({ label, value }: { label: string; value: string }) => {
  return (
    <div className="rounded-sm border border-border/60 bg-background/70 p-3">
      <p className="text-xs uppercase tracking-[0.18em] text-muted-foreground">{label}</p>
      <p className="mt-2 break-all text-sm text-muted-foreground">{value}</p>
    </div>
  )
}

const formatAuthState = (value: string) => value.replaceAll('_', ' ')

const formatExpiry = (value?: bigint | number | null) => {
  if (!value) return 'No expiry recorded.'
  return new Date(Number(value) * 1000).toLocaleString()
}
