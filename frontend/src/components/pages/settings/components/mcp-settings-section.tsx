import { PlusIcon, ServerIcon } from 'lucide-react'
import { observer } from 'mobx-react-lite'
import { useEffect, useState } from 'react'
import { McpServerSheet } from '@/components/forms/mcp-server/mcp-server-sheet'
import { AppLoader } from '@/components/organisms/app-loader'
import { Button } from '@/components/ui/button'
import { Card, CardContent } from '@/components/ui/card'
import { McpSettingsViewmodel, McpSettingsViewmodelContext } from '../mcp-settings.viewmodel'
import { McpServerCard } from './mcp-server-card'

export const McpSettingsSection = observer(() => {
  const [viewmodel] = useState(() => new McpSettingsViewmodel())

  useEffect(() => {
    void viewmodel.init()
  }, [viewmodel])

  if (viewmodel.isLoading) return <AppLoader />

  return (
    <McpSettingsViewmodelContext.Provider value={viewmodel}>
      <div className="flex flex-col gap-4">
        {viewmodel.errorMessage ? (
          <Card>
            <CardContent className="py-4 text-sm text-destructive">{viewmodel.errorMessage}</CardContent>
          </Card>
        ) : null}

        <Card>
          <CardContent className="flex flex-col gap-4 py-4 md:flex-row md:items-center md:justify-between">
            <div className="space-y-1">
              <div className="flex items-center gap-2 font-medium">
                <ServerIcon className="size-4" />
                MCP servers
              </div>
              <p className="text-sm text-muted-foreground">
                Configure global servers and keep OAuth connection state visible before runs.
              </p>
            </div>

            <div className="flex flex-col gap-3 md:flex-row md:items-center">
              <Button type="button" onClick={viewmodel.openCreate}>
                <PlusIcon className="size-4" />
                New server
              </Button>
            </div>
          </CardContent>
        </Card>

        {viewmodel.servers.length === 0 ? (
          <Card>
            <CardContent className="py-8 text-sm text-muted-foreground">No MCP servers configured yet.</CardContent>
          </Card>
        ) : (
          <div className="grid gap-4 xl:grid-cols-2">
            {viewmodel.servers.map((server) => (
              <McpServerCard key={server.id} server={server} />
            ))}
          </div>
        )}

        <McpServerSheet viewmodel={viewmodel.sheet} />
      </div>
    </McpSettingsViewmodelContext.Provider>
  )
})
