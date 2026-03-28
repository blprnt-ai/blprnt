import { Button } from '@/components/ui/button'
import { useProjectViewmodel } from '../project.viewmodel'

export const ProjectHeader = () => {
  const viewmodel = useProjectViewmodel()
  const { project } = viewmodel

  if (!project) return null

  return (
    <div className="flex flex-wrap items-center justify-between gap-3">
      <div>
        <h1 className="text-2xl font-medium">{project.name || 'Untitled project'}</h1>
        <p className="text-sm text-muted-foreground">
          Review project identity, working directories, and recent metadata.
        </p>
      </div>

      <div className="flex items-center gap-2">
        {viewmodel.isEditing ? (
          <>
            <Button type="button" variant="ghost" onClick={viewmodel.cancelEditing}>
              Cancel
            </Button>
            <Button disabled={!viewmodel.canSave} type="button" onClick={() => void viewmodel.save()}>
              {viewmodel.isSaving ? 'Saving...' : 'Save changes'}
            </Button>
          </>
        ) : (
          <Button type="button" onClick={viewmodel.startEditing}>
            Edit project
          </Button>
        )}
      </div>
    </div>
  )
}
