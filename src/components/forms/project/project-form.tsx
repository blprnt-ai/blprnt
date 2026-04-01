import type { FormEvent } from 'react'
import { observer } from 'mobx-react-lite'
import { Button } from '@/components/ui/button'
import { Sheet, SheetContent, SheetDescription, SheetFooter, SheetHeader, SheetTitle } from '@/components/ui/sheet'
import type { ProjectFormViewmodel } from './project-form.viewmodel'
import { ProjectFormFields } from './project-form-fields'

interface ProjectFormProps {
  viewmodel: ProjectFormViewmodel
}

export const ProjectForm = observer(({ viewmodel }: ProjectFormProps) => {
  const handleSubmit = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault()
    await viewmodel.save()
  }

  return (
    <Sheet open={viewmodel.isOpen} onOpenChange={viewmodel.setOpen}>
      <SheetContent
        className="inset-y-0 right-0 h-[100dvh] gap-0 rounded-none border-l border-border p-0 data-[side=right]:left-0 data-[side=right]:w-screen data-[side=right]:max-w-none sm:data-[side=right]:left-auto sm:data-[side=right]:w-full sm:data-[side=right]:max-w-2xl sm:h-full sm:ring-1"
        showCloseButton={!viewmodel.isSaving}
      >
        <form className="flex h-full flex-col" onSubmit={(event) => void handleSubmit(event)}>
          <SheetHeader>
            <SheetTitle>New project</SheetTitle>
            <SheetDescription>Create a project and jump into its workspace settings.</SheetDescription>
          </SheetHeader>

          <ProjectFormFields viewmodel={viewmodel} />

          <SheetFooter className="border-t">
            <Button disabled={viewmodel.isSaving} type="button" variant="ghost" onClick={viewmodel.close}>
              Cancel
            </Button>
            <Button disabled={!viewmodel.canSave} type="submit">
              {viewmodel.isSaving ? 'Creating...' : 'Create project'}
            </Button>
          </SheetFooter>
        </form>
      </SheetContent>
    </Sheet>
  )
})
