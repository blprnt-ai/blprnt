import { observer } from 'mobx-react-lite'
import type { FormEvent } from 'react'
import { Button } from '@/components/ui/button'
import { Sheet, SheetContent, SheetDescription, SheetFooter, SheetHeader, SheetTitle } from '@/components/ui/sheet'
import { MinionFields } from './minion-fields'
import type { MinionSheetViewmodel } from './minion-sheet.viewmodel'

interface MinionSheetProps {
  viewmodel: MinionSheetViewmodel
}

export const MinionSheet = observer(({ viewmodel }: MinionSheetProps) => {
  const handleSubmit = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault()
    await viewmodel.save()
  }

  return (
    <Sheet open={viewmodel.isOpen} onOpenChange={viewmodel.setOpen}>
      <SheetContent
        className="inset-y-0 right-0 h-[100dvh] gap-0 rounded-none border-l border-border p-0 data-[side=right]:left-0 data-[side=right]:w-screen data-[side=right]:max-w-none sm:data-[side=right]:left-auto sm:data-[side=right]:w-full sm:data-[side=right]:max-w-2xl sm:h-full sm:ring-1"
        showCloseButton={!viewmodel.editor.isSaving}
      >
        <form className="flex h-full flex-col" onSubmit={(event) => void handleSubmit(event)}>
          <SheetHeader>
            <SheetTitle>{viewmodel.title}</SheetTitle>
            <SheetDescription>{viewmodel.description}</SheetDescription>
          </SheetHeader>

          <div className="flex-1 overflow-y-auto px-6 py-6">
            <MinionFields minion={viewmodel.editor.minion} />
          </div>

          <SheetFooter className="border-t">
            <Button disabled={viewmodel.editor.isSaving} type="button" variant="ghost" onClick={viewmodel.close}>
              {viewmodel.editor.minion.isReadOnly ? 'Close' : 'Cancel'}
            </Button>
            {viewmodel.editor.minion.isReadOnly ? null : (
              <Button disabled={!viewmodel.editor.canSave} type="submit">
                {viewmodel.editor.isSaving ? 'Saving...' : viewmodel.actionLabel}
              </Button>
            )}
          </SheetFooter>
        </form>
      </SheetContent>
    </Sheet>
  )
})