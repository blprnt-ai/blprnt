import { useState } from 'react'
import type { MinionDto } from '@/bindings/MinionDto'
import { ConfirmationDialog } from '@/components/molecules/confirmation-dialog'
import { Button } from '@/components/ui/button'
import { Card, CardContent } from '@/components/ui/card'

interface MinionCardProps {
  isDeleting: boolean
  minion: MinionDto
  onDelete: () => Promise<void> | void
  onOpen: () => void
}

export const MinionCard = ({ isDeleting, minion, onDelete, onOpen }: MinionCardProps) => {
  const [isConfirmationOpen, setIsConfirmationOpen] = useState(false)

  return (
    <>
      <Card className="border-border/60 py-0">
        <CardContent className="flex h-full flex-col gap-5 px-5 py-5">
          <div className="flex items-start justify-between gap-3">
            <div className="min-w-0 space-y-2">
              <div className="font-medium">{minion.display_name}</div>
              <p className="text-sm text-muted-foreground">{minion.description}</p>
            </div>
            <div className="flex flex-col items-end gap-2 text-xs">
              <span className={minion.enabled ? 'rounded-full bg-primary/10 px-2 py-1 text-primary' : 'rounded-full bg-muted px-2 py-1 text-muted-foreground'}>
                {minion.enabled ? 'Enabled' : 'Disabled'}
              </span>
              <span className="rounded-full bg-muted px-2 py-1 text-muted-foreground">
                {minion.source === 'system' ? 'System' : 'Custom'}
              </span>
            </div>
          </div>

          <div className="grid gap-3 text-sm">
            <div className="rounded-sm border border-border/60 bg-background/75 p-3">
              <p className="text-xs uppercase tracking-[0.18em] text-muted-foreground">Slug</p>
              <p className="mt-2 break-all text-muted-foreground">{minion.slug}</p>
            </div>

            <div className="rounded-sm border border-border/60 bg-background/75 p-3">
              <p className="text-xs uppercase tracking-[0.18em] text-muted-foreground">Prompt</p>
              <p className="mt-2 text-muted-foreground">
                {minion.prompt?.trim().length ? 'Configured' : minion.source === 'system' ? 'Built-in runtime prompt' : 'Not set'}
              </p>
            </div>
          </div>

          <div className="mt-auto flex items-center justify-between gap-3">
            <div>
              {minion.editable ? (
                <Button disabled={isDeleting} size="sm" type="button" variant="destructive-outline" onClick={() => setIsConfirmationOpen(true)}>
                  {isDeleting ? 'Removing...' : 'Delete'}
                </Button>
              ) : null}
            </div>

            <Button size="sm" type="button" variant={minion.editable ? 'outline' : 'secondary'} onClick={onOpen}>
              {minion.editable ? 'Manage' : 'View'}
            </Button>
          </div>
        </CardContent>
      </Card>

      <ConfirmationDialog
        cancelLabel="Keep minion"
        confirmLabel="Delete minion"
        description={`${minion.display_name} will be permanently removed.`}
        open={isConfirmationOpen}
        title={`Delete ${minion.display_name}?`}
        onConfirm={() => {
          setIsConfirmationOpen(false)
          void onDelete()
        }}
        onOpenChange={setIsConfirmationOpen}
      />
    </>
  )
}