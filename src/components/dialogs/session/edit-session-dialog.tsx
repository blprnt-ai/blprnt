import { useEffect, useState } from 'react'
import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle } from '@/components/atoms/dialog'
import { editSessionToast as toast } from '@/components/atoms/toaster'
import { SessionForm } from '@/components/forms/session/session-form'
import { SessionFormViewModel } from '@/components/forms/session/session-form.viewmodel'
import { EventType, globalEventBus } from '@/lib/events'
import { SessionModel } from '@/lib/models/session.model'

interface EditSessionDialogProps {
  sessionId: string
  isOpen: boolean
  onOpenChange: (open: boolean) => void
}

export const EditSessionDialog = ({ sessionId, isOpen, onOpenChange }: EditSessionDialogProps) => {
  const [session, setSession] = useState<SessionFormViewModel | null>(null)

  useEffect(() => {
    if (!isOpen) return
    let isMounted = true
    SessionModel.get(sessionId)
      .then(async (model) => {
        if (!isMounted) return

        setSession(new SessionFormViewModel(model))
      })
      .catch((error) => {
        console.error('Error loading session', error)
      })

    return () => {
      isMounted = false
    }
  }, [sessionId, isOpen])

  if (!session || !isOpen) return null

  const handleSubmit = async () => {
    if (!session?.isValid) return

    toast.loading({ title: 'Updating session...' })
    await session.model.update({
      model_override: session.modelOverride,
      name: session.name,
      network_access: session.networkAccess,
      personality_key: session.model.personalityId,
      queue_mode: session.queueMode,
      read_only: session.readOnly,
      web_search_enabled: session.webSearchEnabled,
      yolo: session.yolo,
    })

    globalEventBus.emit(EventType.Internal, {
      event: { modelOverride: session.modelOverride, sessionId, type: 'model_override_changed' },
    })

    toast.success({ title: 'Session updated successfully' })

    onOpenChange(false)
  }

  return (
    <Dialog open={isOpen} onOpenChange={onOpenChange}>
      <DialogContent size="md">
        <DialogHeader>
          <DialogTitle>Edit Session</DialogTitle>
          <DialogDescription asChild>
            <div className="flex-row gap-8">
              <div>Configure and edit the blprnt session</div>
            </div>
          </DialogDescription>
        </DialogHeader>

        <SessionForm session={session} onSubmit={handleSubmit} />
      </DialogContent>
    </Dialog>
  )
}
