import { AlertCircle, FileQuestion, FileText, RefreshCw } from 'lucide-react'
import { observer } from 'mobx-react-lite'
import { MarkdownEditorPreview } from '@/components/organisms/markdown-editor'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Skeleton } from '@/components/ui/skeleton'
import { useProjectViewmodel } from '../project.viewmodel'

export const ProjectMemoryViewer = observer(() => {
  const viewmodel = useProjectViewmodel()

  if (!viewmodel.selectedMemoryPath) {
    return <ViewerState icon={FileText} message="Select a memory file to read it here." title="No file selected" />
  }

  if (viewmodel.isMemoryFileLoading) {
    return (
      <Card className="border-border/60">
        <CardHeader>
          <CardTitle className="text-base">{viewmodel.selectedMemoryPath}</CardTitle>
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

  if (viewmodel.memoryFileErrorMessage) {
    return (
      <ViewerState
        action={
          <Button type="button" variant="outline" onClick={() => void viewmodel.selectMemoryPath(viewmodel.selectedMemoryPath!)}>
            <RefreshCw className="size-4" />
            Retry
          </Button>
        }
        icon={AlertCircle}
        message={viewmodel.memoryFileErrorMessage}
        title="Could not load file"
      />
    )
  }

  if (!viewmodel.memoryFile) {
    return null
  }

  if (!viewmodel.canPreviewSelectedMemoryFile) {
    return (
      <ViewerState
        icon={FileQuestion}
        message={`Preview is unavailable for ${viewmodel.selectedMemoryFileName ?? 'this file'}.`}
        title="Unsupported preview"
      />
    )
  }

  return (
    <Card className="border-border/60">
      <CardHeader>
        <CardTitle className="text-base">{viewmodel.memoryFile.path}</CardTitle>
      </CardHeader>
      <CardContent>
        {viewmodel.selectedMemoryFileType === 'markdown' ? (
          <div className="min-h-[420px] rounded-md border border-border/60 bg-background/70 px-4 py-3">
            <MarkdownEditorPreview value={viewmodel.memoryFile.content} />
          </div>
        ) : (
          <pre className="min-h-[420px] overflow-x-auto rounded-md border border-border/60 bg-background/70 px-4 py-3 text-sm whitespace-pre-wrap break-words">
            {viewmodel.memoryFile.content}
          </pre>
        )}
      </CardContent>
    </Card>
  )
})

const ViewerState = ({
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
      <CardContent className="flex min-h-[420px] flex-col items-center justify-center gap-3 px-6 py-10 text-center">
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