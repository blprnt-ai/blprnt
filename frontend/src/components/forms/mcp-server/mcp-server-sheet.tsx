import { Loader2 } from 'lucide-react'
import { observer } from 'mobx-react-lite'
import { Button } from '@/components/ui/button'
import { Sheet, SheetContent, SheetDescription, SheetFooter, SheetHeader, SheetTitle } from '@/components/ui/sheet'
import { McpServerFields } from './mcp-server-fields'
import type { McpServerSheetViewmodel } from './mcp-server-sheet.viewmodel'

interface McpServerSheetProps {
  viewmodel: McpServerSheetViewmodel
}

export const McpServerSheet = observer(({ viewmodel }: McpServerSheetProps) => {
  return (
    <Sheet open={viewmodel.isOpen} onOpenChange={viewmodel.setOpen}>
      <SheetContent className="sm:max-w-xl">
        <div className="flex h-full flex-col gap-5">
          <SheetHeader>
            <SheetTitle>{viewmodel.title}</SheetTitle>
            <SheetDescription>{viewmodel.description}</SheetDescription>
          </SheetHeader>

          <form
            className="flex min-h-0 flex-1 flex-col gap-5"
            onSubmit={(event) => {
              event.preventDefault()
              void viewmodel.save()
            }}
          >
            <div className="min-h-0 flex-1 overflow-y-auto px-1">
              <McpServerFields form={viewmodel.form} />
            </div>

            <SheetFooter className="border-t">
              <Button disabled={!viewmodel.form.isValid || viewmodel.isSaving} type="submit">
                {viewmodel.isSaving ? <Loader2 className="size-4 animate-spin" /> : null}
                {viewmodel.actionLabel}
              </Button>
            </SheetFooter>
          </form>
        </div>
      </SheetContent>
    </Sheet>
  )
})
