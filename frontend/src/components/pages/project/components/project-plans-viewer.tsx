import { AlertCircle, FileQuestion, FileText, RefreshCw } from 'lucide-react'
import { observer } from 'mobx-react-lite'
import { MarkdownEditorPreview } from '@/components/organisms/markdown-editor'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Skeleton } from '@/components/ui/skeleton'
import { useProjectViewmodel } from '../project.viewmodel'
import { ProjectViewState } from './project-view-state'

export const ProjectPlansViewer = observer(() => {
  const viewmodel = useProjectViewmodel()

  if (!viewmodel.selectedPlanPath) {
    return (
      <ProjectViewState
        icon={FileText}
        message="Select a plan to read it here."
        minHeight="min-h-[420px]"
        title="No plan selected"
      />
    )
  }

  if (viewmodel.isPlanFileLoading) {
    return (
      <Card className="border-border/60">
        <CardHeader>
          <CardTitle className="text-base">{viewmodel.selectedPlanPath}</CardTitle>
        </CardHeader>
        <CardContent className="space-y-3">
          <Skeleton className="h-4 w-2/3" />
          <Skeleton className="h-4 w-full" />
          <Skeleton className="h-4 w-full" />
          <Skeleton className="h-4 w-5/6" />
        </CardContent>
      </Card>
    )
  }

  if (viewmodel.planFileErrorMessage) {
    return (
      <ProjectViewState
        action={
          <Button
            type="button"
            variant="outline"
            onClick={() => void viewmodel.selectPlanPath(viewmodel.selectedPlanPath!)}
          >
            <RefreshCw className="size-4" />
            Retry
          </Button>
        }
        icon={AlertCircle}
        message={viewmodel.planFileErrorMessage}
        minHeight="min-h-[420px]"
        title="Could not load plan"
      />
    )
  }

  if (!viewmodel.planFile) {
    return null
  }

  if (!viewmodel.canPreviewSelectedPlanFile) {
    return (
      <ProjectViewState
        icon={FileQuestion}
        message={`Preview is unavailable for ${viewmodel.selectedPlanFileName ?? 'this file'}.`}
        minHeight="min-h-[420px]"
        title="Unsupported preview"
      />
    )
  }

  return (
    <Card className="border-border/60">
      <CardHeader>
        <CardTitle className="text-base">{viewmodel.planFile.path}</CardTitle>
      </CardHeader>
      <CardContent>
        {viewmodel.selectedPlanFileType === 'markdown' ? (
          <div className="min-h-[420px] rounded-md border border-border/60 bg-background/70 px-4 py-3">
            <MarkdownEditorPreview value={viewmodel.selectedPlanContent} />
          </div>
        ) : (
          <pre className="min-h-[420px] overflow-x-auto rounded-md border border-border/60 bg-background/70 px-4 py-3 text-sm whitespace-pre-wrap break-words">
            {viewmodel.selectedPlanContent}
          </pre>
        )}
      </CardContent>
    </Card>
  )
})
