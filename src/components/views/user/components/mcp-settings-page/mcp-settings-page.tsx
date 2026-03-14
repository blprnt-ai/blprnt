import { CircleDot, Pencil, Plus, RefreshCw, TestTube2, Trash2 } from 'lucide-react'
import { observer } from 'mobx-react-lite'
import { useEffect, useMemo } from 'react'
import type { McpServerLifecycleState } from '@/bindings'
import { Button } from '@/components/atoms/button'
import { Field, FieldError, FieldGroup, FieldLabel } from '@/components/atoms/field'
import { Input } from '@/components/atoms/input'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/atoms/select'
import { Switch } from '@/components/atoms/switch'
import { Section } from '@/components/organisms/page/section'
import { SectionField } from '@/components/organisms/page/section-field'
import { cn } from '@/lib/utils/cn'
import { McpSettingsViewModel } from './mcp-settings.viewmodel'

const stateBadgeClass = (state: McpServerLifecycleState) => {
  switch (state) {
    case 'connected':
      return 'border-emerald-500/30 bg-emerald-500/10 text-emerald-300'
    case 'connecting':
      return 'border-blue-500/30 bg-blue-500/10 text-blue-300'
    case 'degraded':
      return 'border-amber-500/30 bg-amber-500/10 text-amber-300'
    case 'error':
      return 'border-destructive/30 bg-destructive/10 text-destructive'
    case 'disconnected':
      return 'border-zinc-500/30 bg-zinc-500/10 text-zinc-300'
    case 'configured':
      return 'border-border/70 bg-accent text-muted-foreground'
  }
}

export const McpSettingsPage = observer(() => {
  const viewmodel = useMemo(() => new McpSettingsViewModel(), [])

  useEffect(() => {
    void viewmodel.init()
    return () => viewmodel.destroy()
  }, [viewmodel])

  const selectedServer = viewmodel.selectedServer

  return (
    <Section>
      <SectionField
        title={
          <div className="flex flex-col gap-2">
            <div>MCP Servers</div>
            <div className="flex flex-col gap-1 text-muted-foreground text-sm font-light">
              <div>Add and manage MCP servers used by agents and subagents.</div>
            </div>
          </div>
        }
      >
        <div className="w-full space-y-4">
          <div className="flex items-center gap-2">
            <Button size="sm" variant="outline" onClick={viewmodel.startCreate}>
              <Plus className="size-4" /> Add Server
            </Button>
            <Button
              disabled={viewmodel.isLoading}
              size="sm"
              variant="ghost"
              onClick={() => void viewmodel.refreshAll()}
            >
              <RefreshCw className={cn('size-4', viewmodel.isLoading && 'animate-spin')} /> Refresh
            </Button>
          </div>

          <div className="grid gap-4 lg:grid-cols-[320px_minmax(0,1fr)]">
            <div className="space-y-2 rounded-md border border-border/60 bg-accent/20 p-3">
              {viewmodel.loadError && <div className="text-xs text-destructive">{viewmodel.loadError}</div>}
              {!viewmodel.hasServers && !viewmodel.isCreatingNew && !viewmodel.isLoading && (
                <div className="text-xs text-muted-foreground">No MCP servers configured.</div>
              )}

              {viewmodel.servers.map((server) => {
                const status = viewmodel.statusFor(server.id)
                const state = status?.state ?? 'configured'
                const selected = viewmodel.selectedServerId === server.id && !viewmodel.isCreatingNew
                return (
                  <div
                    key={server.id}
                    role="button"
                    className={cn(
                      'w-full rounded-md border px-2.5 py-2 text-left transition-colors',
                      selected
                        ? 'border-primary/50 bg-primary/10'
                        : 'border-border/60 bg-background/40 hover:border-primary/30 hover:bg-accent/40',
                    )}
                    onClick={() => viewmodel.selectServer(server.id)}
                  >
                    <div className="flex items-center justify-between gap-2">
                      <span className="truncate text-sm font-medium">{server.name}</span>
                      <Switch
                        checked={server.enabled}
                        onCheckedChange={(checked) => void viewmodel.toggleEnabled(server.id, checked)}
                        onClick={(event) => event.stopPropagation()}
                      />
                    </div>
                    <div className="mt-1.5 flex items-center gap-2 text-[11px]">
                      <span
                        className={cn(
                          'inline-flex items-center gap-1 rounded-full border px-2 py-0.5',
                          stateBadgeClass(state),
                        )}
                      >
                        <CircleDot className={cn('size-2', state === 'connected' && 'animate-pulse')} />
                        {viewmodel.stateLabel(state)}
                      </span>
                      <span className="rounded-full border border-border/70 px-2 py-0.5 text-muted-foreground">
                        {server.transport.type}
                      </span>
                    </div>
                    {status?.error && (
                      <div className="mt-1 text-[11px] text-muted-foreground line-clamp-2">{status.error}</div>
                    )}
                  </div>
                )
              })}
            </div>

            <div className="space-y-4 rounded-md border border-border/60 bg-accent/20 p-4">
              <div className="flex items-center justify-between gap-2">
                <h3 className="text-sm font-semibold">
                  {viewmodel.isCreatingNew ? 'New MCP Server' : selectedServer ? 'Edit MCP Server' : 'Select a Server'}
                </h3>
                {(viewmodel.isCreatingNew || selectedServer) && (
                  <div className="flex items-center gap-1">
                    <Button
                      disabled={!selectedServer || viewmodel.isTesting || !viewmodel.enabled}
                      size="icon-xs"
                      variant="ghost"
                      onClick={() => void viewmodel.testSelectedConnection()}
                    >
                      <TestTube2 className="size-4" />
                    </Button>
                    <Button
                      disabled={!selectedServer || viewmodel.isDeleting}
                      size="icon-xs"
                      variant="ghost"
                      onClick={() => void viewmodel.deleteSelected()}
                    >
                      <Trash2 className="size-4" />
                    </Button>
                  </div>
                )}
              </div>

              {viewmodel.isCreatingNew || selectedServer ? (
                <FieldGroup>
                  {viewmodel.submitError && (
                    <div className="rounded-md border border-destructive/30 bg-destructive/10 px-3 py-2 text-xs text-destructive">
                      {viewmodel.submitError}
                    </div>
                  )}

                  <Field>
                    <FieldLabel>Name</FieldLabel>
                    <Input value={viewmodel.name} onChange={(event) => viewmodel.setName(event.target.value)} />
                    <FieldError>{viewmodel.fieldErrors.get('name')}</FieldError>
                  </Field>

                  <Field>
                    <FieldLabel>Enabled</FieldLabel>
                    <div className="flex items-center gap-2">
                      <Switch checked={viewmodel.enabled} onCheckedChange={viewmodel.setEnabled} />
                      <span className="text-xs text-muted-foreground">Enable runtime connection and tool usage.</span>
                    </div>
                  </Field>

                  <Field>
                    <FieldLabel>Transport</FieldLabel>
                    <Select
                      value={viewmodel.transportType}
                      onValueChange={(value) => viewmodel.setTransportType(value as 'stdio' | 'sse_http')}
                    >
                      <SelectTrigger className="w-52">
                        <SelectValue />
                      </SelectTrigger>
                      <SelectContent>
                        <SelectItem value="stdio">stdio</SelectItem>
                        <SelectItem value="sse_http">sse/http</SelectItem>
                      </SelectContent>
                    </Select>
                  </Field>

                  {viewmodel.transportType === 'stdio' ? (
                    <>
                      <Field>
                        <FieldLabel>Command</FieldLabel>
                        <Input
                          value={viewmodel.stdioCommand}
                          onChange={(event) => viewmodel.setStdioCommand(event.target.value)}
                        />
                        <FieldError>{viewmodel.fieldErrors.get('stdioCommand')}</FieldError>
                      </Field>

                      <Field>
                        <FieldLabel>Args (space separated)</FieldLabel>
                        <Input
                          placeholder="-y @modelcontextprotocol/server-filesystem /workspace"
                          value={viewmodel.stdioArgs}
                          onChange={(event) => viewmodel.setStdioArgs(event.target.value)}
                        />
                      </Field>

                      <Field>
                        <FieldLabel>Working Directory (optional)</FieldLabel>
                        <Input
                          value={viewmodel.stdioCwd}
                          onChange={(event) => viewmodel.setStdioCwd(event.target.value)}
                        />
                      </Field>

                      <Field>
                        <FieldLabel>Environment (KEY=VALUE per line)</FieldLabel>
                        <textarea
                          className="min-h-24 w-full rounded-md border border-input bg-accent px-3 py-2 text-sm"
                          value={viewmodel.stdioEnv}
                          onChange={(event) => viewmodel.setStdioEnv(event.target.value)}
                        />
                        <FieldError>{viewmodel.fieldErrors.get('stdioEnv')}</FieldError>
                      </Field>
                    </>
                  ) : (
                    <Field>
                      <FieldLabel>URL</FieldLabel>
                      <Input value={viewmodel.sseUrl} onChange={(event) => viewmodel.setSseUrl(event.target.value)} />
                      <FieldError>{viewmodel.fieldErrors.get('sseUrl')}</FieldError>
                    </Field>
                  )}

                  <Field>
                    <FieldLabel>Authentication</FieldLabel>
                    <Select
                      value={viewmodel.authType}
                      onValueChange={(value) =>
                        viewmodel.setAuthType(value as 'none' | 'bearer_token' | 'api_key' | 'basic' | 'headers')
                      }
                    >
                      <SelectTrigger className="w-52">
                        <SelectValue />
                      </SelectTrigger>
                      <SelectContent>
                        <SelectItem value="none">none</SelectItem>
                        <SelectItem value="bearer_token">bearer token</SelectItem>
                        <SelectItem value="api_key">api key</SelectItem>
                        <SelectItem value="basic">basic</SelectItem>
                        <SelectItem value="headers">headers</SelectItem>
                      </SelectContent>
                    </Select>
                  </Field>

                  {viewmodel.authType === 'bearer_token' && (
                    <Field>
                      <FieldLabel>Bearer Token</FieldLabel>
                      <Input
                        type="password"
                        value={viewmodel.bearerToken}
                        onChange={(event) => viewmodel.setBearerToken(event.target.value)}
                      />
                      <FieldError>{viewmodel.fieldErrors.get('bearerToken')}</FieldError>
                    </Field>
                  )}

                  {viewmodel.authType === 'api_key' && (
                    <>
                      <Field>
                        <FieldLabel>API Key Header</FieldLabel>
                        <Input
                          value={viewmodel.apiKeyHeader}
                          onChange={(event) => viewmodel.setApiKeyHeader(event.target.value)}
                        />
                        <FieldError>{viewmodel.fieldErrors.get('apiKeyHeader')}</FieldError>
                      </Field>
                      <Field>
                        <FieldLabel>API Key Value</FieldLabel>
                        <Input
                          type="password"
                          value={viewmodel.apiKeyValue}
                          onChange={(event) => viewmodel.setApiKeyValue(event.target.value)}
                        />
                        <FieldError>{viewmodel.fieldErrors.get('apiKeyValue')}</FieldError>
                      </Field>
                    </>
                  )}

                  {viewmodel.authType === 'basic' && (
                    <>
                      <Field>
                        <FieldLabel>Username</FieldLabel>
                        <Input
                          value={viewmodel.basicUsername}
                          onChange={(event) => viewmodel.setBasicUsername(event.target.value)}
                        />
                        <FieldError>{viewmodel.fieldErrors.get('basicUsername')}</FieldError>
                      </Field>
                      <Field>
                        <FieldLabel>Password</FieldLabel>
                        <Input
                          type="password"
                          value={viewmodel.basicPassword}
                          onChange={(event) => viewmodel.setBasicPassword(event.target.value)}
                        />
                        <FieldError>{viewmodel.fieldErrors.get('basicPassword')}</FieldError>
                      </Field>
                    </>
                  )}

                  {viewmodel.authType === 'headers' && (
                    <Field>
                      <FieldLabel>Auth Headers (KEY=VALUE per line)</FieldLabel>
                      <textarea
                        className="min-h-24 w-full rounded-md border border-input bg-accent px-3 py-2 text-sm"
                        value={viewmodel.authHeaders}
                        onChange={(event) => viewmodel.setAuthHeaders(event.target.value)}
                      />
                      <FieldError>{viewmodel.fieldErrors.get('authHeaders')}</FieldError>
                    </Field>
                  )}

                  <div className="flex items-center gap-2">
                    <Button disabled={viewmodel.isSaving} variant="outline" onClick={() => void viewmodel.save()}>
                      {viewmodel.isSaving ? (
                        <RefreshCw className="size-4 animate-spin" />
                      ) : viewmodel.isCreatingNew ? (
                        <Plus className="size-4" />
                      ) : (
                        <Pencil className="size-4" />
                      )}
                      {viewmodel.isCreatingNew ? 'Create Server' : 'Save Changes'}
                    </Button>
                    <Button
                      disabled={!selectedServer || viewmodel.isTesting || !viewmodel.enabled}
                      variant="ghost"
                      onClick={() => void viewmodel.testSelectedConnection()}
                    >
                      {viewmodel.isTesting ? (
                        <RefreshCw className="size-4 animate-spin" />
                      ) : (
                        <TestTube2 className="size-4" />
                      )}{' '}
                      Test Connection
                    </Button>
                  </div>
                  {selectedServer && !viewmodel.enabled && (
                    <div className="text-xs text-muted-foreground">Enable this server to run Test Connection.</div>
                  )}
                </FieldGroup>
              ) : (
                <div className="text-xs text-muted-foreground">
                  Select a server or create a new one to edit settings.
                </div>
              )}
            </div>
          </div>
        </div>
      </SectionField>
    </Section>
  )
})
