import { AlertCircle, FolderOpen, RefreshCw } from 'lucide-react'
import { observer } from 'mobx-react-lite'
import { Button } from '@/components/ui/button'
import { Card, CardContent } from '@/components/ui/card'
import { ProjectMemoryTree } from './project-memory-tree'
import { ProjectMemoryViewer } from './project-memory-viewer'
import { useProjectViewmodel } from '../project.viewmodel'

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
      <StateCard
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
    return <StateCard icon={FolderOpen} message="This project does not have any memory files yet." title="No memory files" />
  }

  return (
    <div className="grid gap-4 xl:grid-cols-[280px_minmax(0,1fr)]">
      <ProjectMemoryTree />
      <ProjectMemoryViewer />
    </div>
  )
})

const StateCard = ({
  action,
  icon: Icon,
  message,
  title,
}: {
  action?: React.ReactNode
  icon: React.ComponentType<{ className?: string }>
  message: string
  title: string
}) => {
  return (
    <Card className="border-border/60">
      <CardContent className="flex min-h-[320px] flex-col items-center justify-center gap-3 px-6 py-10 text-center">
        <div className="flex size-12 items-center justify-center rounded-full border border-border/60 bg-muted/30">
          <Icon className="size-5 text-muted-foreground" />
        </div>
        <div className="space-y-1">
          <h3 className="text-base font-medium">{title}</h3>
          <p className="text-sm text-muted-foreground">{message}</p>
        </div>
        {action}
      </CardContent>
    </Card>
  )
}