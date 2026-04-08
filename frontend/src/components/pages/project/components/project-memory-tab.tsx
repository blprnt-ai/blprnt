import { AlertCircle, FolderOpen, RefreshCw, Search } from 'lucide-react'
import { observer } from 'mobx-react-lite'
import type * as React from 'react'
import { Button } from '@/components/ui/button'
import { Card, CardContent } from '@/components/ui/card'
import { Input } from '@/components/ui/input'
import { getProjectMemoryResultLabel, useProjectViewmodel } from '../project.viewmodel'
import { ProjectMemoryTree } from './project-memory-tree'
import { ProjectMemoryViewer } from './project-memory-viewer'
import { ProjectViewState } from './project-view-state'

export const ProjectMemoryTab = observer(() => {
  const viewmodel = useProjectViewmodel()

  if (viewmodel.isMemoryLoading) {
    return (
      <Card className="border-border/60">
        <CardContent className="py-10 text-sm text-muted-foreground">Loading project memory...</CardContent>
      </Card>
    )
  }

  if (viewmodel.memoryErrorMessage) {
    return (
      <ProjectViewState
        action={
          <Button type="button" variant="outline" onClick={() => void viewmodel.reloadMemoryTree()}>
            <RefreshCw className="size-4" />
            Retry
          </Button>
        }
        icon={AlertCircle}
        message={viewmodel.memoryErrorMessage}
        title="Could not load project memory"
      />
    )
  }

  if (!viewmodel.hasMemoryFiles) {
    return (
      <ProjectViewState
        icon={FolderOpen}
        message="This project does not have any memory files yet."
        title="No memory files"
      />
    )
  }

  return (
    <div className="space-y-4">
      <Card className="border-border/60">
        <CardContent className="space-y-3 p-4">
          <form
            className="flex flex-col gap-3 sm:flex-row"
            onSubmit={(event) => void handleSubmit(event, viewmodel.searchMemory)}
          >
            <div className="relative flex-1">
              <Search className="pointer-events-none absolute left-3 top-1/2 size-4 -translate-y-1/2 text-muted-foreground" />
              <Input
                className="pl-9"
                placeholder="Search project memory"
                value={viewmodel.memorySearchQuery}
                onChange={(event) => viewmodel.setMemorySearchQuery(event.target.value)}
              />
            </div>
            <div className="flex gap-2">
              <Button disabled={!viewmodel.hasMemorySearchQuery || viewmodel.isMemorySearchLoading} type="submit">
                {viewmodel.isMemorySearchLoading ? 'Searching...' : 'Search'}
              </Button>
              {viewmodel.hasMemorySearchQuery ? (
                <Button type="button" variant="outline" onClick={() => viewmodel.setMemorySearchQuery('')}>
                  Clear
                </Button>
              ) : null}
            </div>
          </form>

          {viewmodel.memorySearchErrorMessage ? (
            <div className="rounded-md border border-destructive/30 bg-destructive/5 px-3 py-2 text-sm text-destructive">
              {viewmodel.memorySearchErrorMessage}
            </div>
          ) : null}

          {viewmodel.hasMemorySearchQuery && !viewmodel.isMemorySearchLoading ? (
            viewmodel.hasMemorySearchResults ? (
              <div className="space-y-2">
                <p className="text-sm text-muted-foreground">
                  Search results open in the existing viewer and keep the current file selection in sync.
                </p>
                <div className="space-y-2">
                  {viewmodel.memorySearchResults.map((result, index) => (
                    <button
                      key={`${result.path ?? result.title}-${index}`}
                      className="w-full rounded-md border border-border/60 px-3 py-3 text-left transition hover:bg-muted/30"
                      type="button"
                      onClick={() => void viewmodel.selectMemorySearchResult(result)}
                    >
                      <div className="flex items-start justify-between gap-3">
                        <div className="min-w-0 space-y-1">
                          <div className="truncate text-sm font-medium">{getProjectMemoryResultLabel(result)}</div>
                          <div className="truncate text-xs text-muted-foreground">{result.path ?? result.title}</div>
                        </div>
                        <div className="shrink-0 text-xs text-muted-foreground">{result.score.toFixed(2)}</div>
                      </div>
                      <p className="mt-2 line-clamp-3 text-sm text-muted-foreground">{result.content}</p>
                    </button>
                  ))}
                </div>
              </div>
            ) : viewmodel.memorySearchErrorMessage ? null : (
              <div className="rounded-md border border-dashed border-border/70 px-3 py-4 text-sm text-muted-foreground">
                No memory files matched “{viewmodel.memorySearchQuery.trim()}”.
              </div>
            )
          ) : null}
        </CardContent>
      </Card>

      <div className="grid gap-4 xl:grid-cols-[280px_minmax(0,1fr)]">
        <ProjectMemoryTree />
        <ProjectMemoryViewer />
      </div>
    </div>
  )
})

const handleSubmit = (event: React.FormEvent<HTMLFormElement>, action: () => Promise<void>) => {
  event.preventDefault()
  void action()
}
