import { useEffect, useState } from 'react'
import { deleteProjectToast as toast } from '@/components/atoms/toaster'
import { DeleteConfirmDialog } from '@/components/dialogs/delete-confirm-dialog'
import { useDockviewLayoutViewModel } from '@/components/dockview/dockview-layout.viewmodel'
import { ProjectModel } from '@/lib/models/project.model'
import { SessionModel } from '@/lib/models/session.model'
import { reportError } from '@/lib/utils/error-reporting'

interface DeleteProjectDialogProps {
  projectId: string
  isOpen: boolean
  onOpenChange: (isOpen: boolean) => void
}

export const DeleteProjectDialog = ({ projectId, isOpen, onOpenChange }: DeleteProjectDialogProps) => {
  const dockviewLayout = useDockviewLayoutViewModel()
  const [project, setProject] = useState<ProjectModel | null>(null)
  const [meta, setMeta] = useState({
    sessionsCounts: 0,
  })

  const handleConfirm = async () => {
    if (!project) return
    try {
      toast.loading({ title: 'Deleting project...' })
      await project.delete()
      toast.success({ title: 'Project deleted successfully' })

      dockviewLayout.closePanelsByPredicate(({ params }) => params.projectId === project.id)
      onOpenChange(false)
    } catch (error) {
      reportError(error, 'deleting project')
      toast.error({ title: `Failed to delete project` })
    }
  }

  const handleCancel = () => onOpenChange(false)

  useEffect(() => {
    let isMounted = true
    const loadMeta = async () => {
      const [projectModel, sessions] = await Promise.all([ProjectModel.get(projectId), SessionModel.list(projectId)])

      if (!isMounted) return

      setProject(projectModel)
      setMeta({
        sessionsCounts: sessions.length,
      })
    }

    loadMeta().catch((error) => {
      console.error('Error loading project meta', error)
    })

    return () => {
      isMounted = false
    }
  }, [projectId])

  if (!project) return null

  const description = (
    <span className="flex flex-col gap-2">
      <span>Are you sure you want to delete this project?</span>
      <span className="text-sm italic text-destructive/80">This action cannot be undone.</span>

      <span className="text-warn/80">
        This project will be removed and all associated sessions ({meta.sessionsCounts}) data will be deleted.
      </span>
    </span>
  )

  return (
    <DeleteConfirmDialog
      description={description}
      isOpen={isOpen}
      title="Delete Project"
      onCancel={handleCancel}
      onConfirm={handleConfirm}
      onOpenChange={onOpenChange}
    />
  )
}
