import { FolderKanban, FolderRoot } from 'lucide-react'
import { Card, CardContent } from '@/components/ui/card'
import { useProjectViewmodel } from '../project.viewmodel'
import { formatDate, formatDirectoryCount } from '../utils'

export const ProjectHeader = () => {
  const viewmodel = useProjectViewmodel()
  const { project } = viewmodel

  if (!project) return null

  const statusItems = [
    formatDirectoryCount(viewmodel.workingDirectoryCount),
    `Updated ${formatDate(project.updatedAt)}`,
  ]

  return (
    <Card className="overflow-hidden border-border/60 bg-gradient-to-br from-card via-card to-muted/30 py-0">
      <CardContent className="px-5 py-6 md:px-6">
        <div className="space-y-4">
          <div className="flex flex-wrap items-start gap-4">
            <div className="flex size-16 items-center justify-center rounded-2xl border border-border/60 bg-background/75 shadow-sm backdrop-blur">
              <FolderKanban className="size-7 text-muted-foreground" />
            </div>
            <div className="min-w-0 flex-1 space-y-2">
              <div className="space-y-1">
                <h1 className="truncate text-3xl font-medium tracking-tight">{project.name || 'Untitled project'}</h1>
                <p className="text-base text-muted-foreground">
                  Define the project identity and the working directories where blprnt should operate.
                </p>
              </div>
              <div className="flex flex-wrap gap-2">
                {statusItems.map((item) => (
                  <span
                    key={item}
                    className="rounded-full border border-border/60 bg-background/70 px-3 py-1 text-xs uppercase tracking-[0.18em] text-muted-foreground"
                  >
                    {item}
                  </span>
                ))}
              </div>
            </div>
          </div>
          <div className="flex items-start gap-2 text-sm text-muted-foreground">
            <FolderRoot className="mt-0.5 size-4 shrink-0" />
            <p className="leading-6">
              This page saves in place. Update the project name or directory list directly without switching into a
              separate edit mode.
            </p>
          </div>
        </div>
      </CardContent>
    </Card>
  )
}
