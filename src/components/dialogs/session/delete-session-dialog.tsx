import { captureException } from '@sentry/react'
import { deleteSessionToast as toast } from '@/components/atoms/toaster'
import { DeleteConfirmDialog } from '@/components/dialogs/delete-confirm-dialog'
import { useDockviewLayoutViewModel } from '@/components/dockview/dockview-layout.viewmodel'
import { SessionModel } from '@/lib/models/session.model'

interface DeleteSessionDialogProps {
  isOpen: boolean
  onOpenChange: (isOpen: boolean) => void
  sessionId: string
}

export const DeleteSessionDialog = ({ isOpen, onOpenChange, sessionId }: DeleteSessionDialogProps) => {
  const dockviewLayout = useDockviewLayoutViewModel()

  const handleConfirm = async () => {
    try {
      toast.loading({ title: 'Deleting session...' })
      await SessionModel.deleteById(sessionId)
      toast.success({ duration: 2500, title: 'Session deleted' })

      dockviewLayout.closePanelsByPredicate(({ params }) => params.sessionId === sessionId)
      onOpenChange(false)
    } catch (error) {
      captureException(error)
      toast.error({ duration: 5000, title: 'Failed to delete session' })
    }
  }

  const handleCancel = () => onOpenChange(false)

  const description = (
    <span className="flex flex-col gap-2">
      <span>Are you sure you want to delete this session?</span>
      <span className="text-sm italic text-destructive/80">This action cannot be undone.</span>
    </span>
  )

  return (
    <DeleteConfirmDialog
      description={description}
      isOpen={isOpen}
      title="Delete Session"
      onCancel={handleCancel}
      onConfirm={handleConfirm}
      onOpenChange={onOpenChange}
    />
  )
}
