import { Save } from 'lucide-react'
import { observer } from 'mobx-react-lite'
import { MarkdownEditor, MarkdownEditorPreview } from '@/components/organisms/markdown-editor'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { useEmployeeViewmodel } from '../employee.viewmodel'

export const EmployeeLifeFilePanel = observer(() => {
  const viewmodel = useEmployeeViewmodel()

  if (viewmodel.isLifeFileLoading) {
    return (
      <Card className="border-border/60">
        <CardContent className="py-6 text-sm text-muted-foreground">Loading file...</CardContent>
      </Card>
    )
  }

  if (!viewmodel.lifeFile) {
    return (
      <Card className="border-border/60">
        <CardContent className="py-6 text-sm text-muted-foreground">Select a file to inspect.</CardContent>
      </Card>
    )
  }

  return (
    <Card className="border-border/60">
      <CardHeader className="flex flex-row items-center justify-between gap-3">
        <CardTitle className="text-base">{viewmodel.lifeFile.path}</CardTitle>
        {viewmodel.canEditSelectedLifeFile ? (
          <Button
            disabled={!viewmodel.hasUnsavedLifeChanges || viewmodel.isLifeSaving}
            size="sm"
            type="button"
            onClick={() => void viewmodel.saveLifeFile()}
          >
            <Save className="size-4" />
            {viewmodel.isLifeSaving ? 'Saving...' : 'Save'}
          </Button>
        ) : null}
      </CardHeader>
      <CardContent>
        {viewmodel.canEditSelectedLifeFile ? (
          <MarkdownEditor
            editorClassName="min-h-[420px]"
            value={viewmodel.lifeDraftContent}
            onChange={viewmodel.setLifeDraftContent}
          />
        ) : (
          <div className="min-h-[420px] rounded-md border border-border/60 bg-background/70">
            <MarkdownEditorPreview className="px-4 py-3" value={viewmodel.lifeFile.content} />
          </div>
        )}
      </CardContent>
    </Card>
  )
})
