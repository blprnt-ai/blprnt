import type { FormEvent } from 'react'
import { Button } from '@/components/ui/button'
import { Sheet, SheetContent, SheetDescription, SheetFooter, SheetHeader, SheetTitle } from '@/components/ui/sheet'
import type { IssueFormViewmodel } from './issue-form.viewmodel'
import { IssueFormFields } from './issue-form-fields'

interface IssueFormProps {
  viewmodel: IssueFormViewmodel
}

export const IssueForm = ({ viewmodel }: IssueFormProps) => {
  const handleSubmit = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault()
    await viewmodel.save()
  }

  return (
    <Sheet open={viewmodel.isOpen} onOpenChange={viewmodel.setOpen}>
      <SheetContent className="w-full gap-0 p-0 sm:max-w-2xl" showCloseButton={!viewmodel.isSaving}>
        <form className="flex h-full flex-col" onSubmit={(event) => void handleSubmit(event)}>
          <SheetHeader>
            <SheetTitle>New issue</SheetTitle>
            <SheetDescription>Create an issue and jump straight into the detail view.</SheetDescription>
          </SheetHeader>

          <IssueFormFields viewmodel={viewmodel} />

          <SheetFooter className="border-t">
            <Button disabled={viewmodel.isSaving} type="button" variant="ghost" onClick={viewmodel.close}>
              Cancel
            </Button>
            <Button disabled={!viewmodel.canSave} type="submit">
              {viewmodel.isSaving ? 'Creating...' : 'Create issue'}
            </Button>
          </SheetFooter>
        </form>
      </SheetContent>
    </Sheet>
  )
}
