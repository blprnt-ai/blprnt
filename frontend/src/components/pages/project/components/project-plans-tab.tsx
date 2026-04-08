import { AlertCircle, FolderOpen, RefreshCw } from 'lucide-react'
import { observer } from 'mobx-react-lite'
import { Button } from '@/components/ui/button'
import { Card, CardContent } from '@/components/ui/card'
import { useProjectViewmodel } from '../project.viewmodel'
import { ProjectPlansList } from './project-plans-list'
import { ProjectPlansViewer } from './project-plans-viewer'
import { ProjectViewState } from './project-view-state'

export const ProjectPlansTab = observer(() => {
  const viewmodel = useProjectViewmodel()

  if (viewmodel.isPlansLoading) {
    return (
      <Card className="border-border/60">
        <CardContent className="py-10 text-sm text-muted-foreground">Loading project plans...</CardContent>
      </Card>
    )
  }

  if (viewmodel.plansErrorMessage) {
    return (
      <ProjectViewState
        action={
          <Button type="button" variant="outline" onClick={() => void viewmodel.reloadPlans()}>
            <RefreshCw className="size-4" />
            Retry
          </Button>
        }
        icon={AlertCircle}
        message={viewmodel.plansErrorMessage}
        title="Could not load project plans"
      />
    )
  }

  if (!viewmodel.hasPlans) {
    return (
      <ProjectViewState icon={FolderOpen} message="This project does not have any plan files yet." title="No plans" />
    )
  }

  return (
    <div className="grid gap-4 xl:grid-cols-[320px_minmax(0,1fr)]">
      <ProjectPlansList />
      <ProjectPlansViewer />
    </div>
  )
})
