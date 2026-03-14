import { captureException } from '@sentry/react'
import { useCallback } from 'react'
import { Button } from '@/components/atoms/button'
import { Dialog, DialogContent, DialogFooter, DialogHeader, DialogTitle } from '@/components/atoms/dialog'
import { editPersonalityToast as toast } from '@/components/atoms/toaster'
import type { PersonalityViewModel } from '@/components/views/personalities/personalities.viewmodel'
import { PersonalityForm } from './personality-form'

interface EditPersonalityDialogProps {
  isOpen: boolean
  onOpenChange: (open: boolean) => void
  personality: PersonalityViewModel
}

export const EditPersonalityDialog = ({ isOpen, onOpenChange, personality }: EditPersonalityDialogProps) => {
  const handleCreate = useCallback(async () => {
    if (!personality?.isValid) return

    toast.loading({ title: 'Updating personality...' })
    try {
      await personality.update()
      toast.success({ title: 'Personality updated successfully' })
      onOpenChange(false)
    } catch (error) {
      captureException(error)
      toast.error({ title: `Failed to update personality: ${error}` })
    }
  }, [personality, onOpenChange])

  return (
    <Dialog open={isOpen} onOpenChange={onOpenChange}>
      <DialogContent size="3xl">
        <DialogHeader>
          <DialogTitle>Edit Personality</DialogTitle>
        </DialogHeader>

        <PersonalityForm personality={personality} />

        <DialogFooter>
          <Button variant="ghost" onClick={() => onOpenChange(false)}>
            Cancel
          </Button>
          <Button disabled={!personality?.isValid} variant="outline" onClick={handleCreate}>
            Update
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
