import { DialogDescription } from '@radix-ui/react-dialog'
import { Trash2 } from 'lucide-react'
import { useMemo, useState } from 'react'
import { Button } from '@/components/atoms/button'
import { Checkbox } from '@/components/atoms/checkbox'
import { Dialog, DialogContent, DialogFooter, DialogHeader, DialogTitle } from '@/components/atoms/dialog'
import { FieldLabel } from '@/components/atoms/field'
import { Input } from '@/components/atoms/input'

interface DeleteConfirmDialogProps {
  isOpen: boolean
  onOpenChange: (isOpen: boolean) => void
  title: string
  description: React.ReactNode
  onConfirm: () => void | Promise<void>
  onCancel: () => void
  requiresForce?: boolean
  forceDelete?: boolean
  setForceDelete?: (forceDelete: boolean) => void
}

const CONFIRMATION_TEXT = 'DELETE'

export const DeleteConfirmDialog = ({
  isOpen,
  onOpenChange,
  title,
  description,
  onConfirm,
  onCancel,
  requiresForce,
  forceDelete,
  setForceDelete,
}: DeleteConfirmDialogProps) => {
  const [confirmation, setConfirmation] = useState('')

  const handleConfirmationChange = (e: React.ChangeEvent<HTMLInputElement>) => setConfirmation(e.target.value)

  const isValid = useMemo(
    () => confirmation === CONFIRMATION_TEXT && (!requiresForce || forceDelete),
    [requiresForce, forceDelete, confirmation],
  )
  const isConfirming = useMemo(() => CONFIRMATION_TEXT.startsWith(confirmation), [confirmation])

  const handleSubmit = async (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault()
    // if (!isValid) return

    await handleConfirm()
  }

  const handleConfirm = async () => await onConfirm()
  const handleCancel = () => onCancel()

  return (
    <Dialog open={isOpen} onOpenChange={onOpenChange}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{title}</DialogTitle>
        </DialogHeader>
        <DialogDescription>
          <span>{description}</span>
        </DialogDescription>

        <form onSubmit={handleSubmit}>
          <div className="flex flex-col gap-2">
            {(!requiresForce || forceDelete) && (
              <>
                <div>
                  Type <span className="font-mono">DELETE</span> to confirm
                </div>
                <div>
                  <Input
                    aria-invalid={!isConfirming}
                    placeholder="DELETE"
                    type="text"
                    value={confirmation}
                    onChange={handleConfirmationChange}
                  />
                </div>
              </>
            )}
            {requiresForce && (
              <div className="flex items-center justify-start gap-2">
                <FieldLabel htmlFor="force-delete">Force Delete</FieldLabel>

                <Checkbox checked={forceDelete} id="force-delete" onCheckedChange={setForceDelete} />
              </div>
            )}

            <DialogFooter>
              <Button type="button" variant="ghost" onClick={handleCancel}>
                Cancel
              </Button>
              <Button disabled={!isValid} type="submit" variant="destructive">
                <Trash2 size={16} />
                Delete
              </Button>
            </DialogFooter>
          </div>
        </form>
      </DialogContent>
    </Dialog>
  )
}
