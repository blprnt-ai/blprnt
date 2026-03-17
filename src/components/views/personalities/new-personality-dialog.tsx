import { useCallback, useMemo } from 'react'
import { Button } from '@/components/atoms/button'
import { Dialog, DialogContent, DialogFooter, DialogHeader, DialogTitle } from '@/components/atoms/dialog'
import { newPersonalityToast as toast } from '@/components/atoms/toaster'
import {
  type PersonalitiesViewModel,
  PersonalityViewModel,
} from '@/components/views/personalities/personalities.viewmodel'
import { PersonalityModel } from '@/lib/models/personality.model'
import { defaultPersonalityModel } from '@/lib/utils/default-models'
import { reportError } from '@/lib/utils/error-reporting'
import { PersonalityForm } from './personality-form'

interface NewPersonalityDialogProps {
  isOpen: boolean
  personalities: PersonalitiesViewModel
  onOpenChange: (open: boolean) => void
  onCreated?: (personality: PersonalityViewModel) => void
}

export const NewPersonalityDialog = ({ isOpen, onOpenChange, personalities, onCreated }: NewPersonalityDialogProps) => {
  const newPersonality = useMemo(
    () => new PersonalityViewModel(personalities, new PersonalityModel({ ...defaultPersonalityModel })),
    [personalities],
  )

  const handleCreate = useCallback(async () => {
    if (!newPersonality?.isValid) return

    toast.loading({ title: 'Creating personality...' })
    try {
      const createdPersonality = await newPersonality.create()
      toast.success({ title: 'Personality created successfully' })
      onCreated?.(createdPersonality)
      onOpenChange(false)
    } catch (error) {
      reportError(error, 'creating personality')
      toast.error({ title: `Failed to create personality: ${error}` })
    }
  }, [newPersonality, onCreated, onOpenChange])

  if (!newPersonality) return null

  return (
    <Dialog open={isOpen} onOpenChange={onOpenChange}>
      <DialogContent size="3xl">
        <DialogHeader>
          <DialogTitle>New Personality</DialogTitle>
        </DialogHeader>

        <PersonalityForm personality={newPersonality} />

        <DialogFooter>
          <Button variant="ghost" onClick={() => onOpenChange(false)}>
            Cancel
          </Button>
          <Button disabled={!newPersonality?.isValid} variant="outline" onClick={handleCreate}>
            Create
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
