import { Button } from '@/components/ui/button'
import { useProjectViewmodel } from '../project.viewmodel'

export const ProjectDirectoryActions = () => {
  const viewmodel = useProjectViewmodel()
  const { project } = viewmodel

  if (!project || !viewmodel.isEditing) return null

  return (
    <div className="flex justify-end">
      <Button type="button" variant="outline" onClick={project.addWorkingDirectory}>
        Add folder
      </Button>
    </div>
  )
}
