import { ArrowLeftIcon, ArrowRightIcon, FolderIcon, TrashIcon } from 'lucide-react'
import { observer } from 'mobx-react-lite'
import { LabeledTextarea } from '@/components/molecules/labeled-textarea'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardFooter } from '@/components/ui/card'
import { Input } from '@/components/ui/input'
import { InputGroup, InputGroupAddon, InputGroupButton, InputGroupInput } from '@/components/ui/input-group'
import { Label } from '@/components/ui/label'
import { OnboardingStep, useOnboardingViewmodel } from './onboarding.viewmodel'
import { OnboardingCardHeader } from './onboarding-card-header'

export const CreateProject = observer(() => {
  const viewmodel = useOnboardingViewmodel()

  const handleNameChange = (value: string) => {
    viewmodel.project.name = value
  }

  const handleAddWorkingDirectory = () => {
    viewmodel.project.addWorkingDirectory()
  }

  const handleDescriptionChange = (value: string) => {
    viewmodel.project.description = value
  }

  const handleRemoveWorkingDirectory = (index: number) => {
    viewmodel.project.removeWorkingDirectory(index)
  }

  const handleWorkingDirectoryChange = (index: number, value: string) => {
    viewmodel.project.setWorkingDirectory(index, value)
  }

  const handleSave = async (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault()

    await viewmodel.saveProject()
  }

  const verb = !viewmodel.project?.id || viewmodel.project?.isDirty ? 'Next' : 'Save'

  return (
    <Card className="w-full">
      <form onSubmit={handleSave}>
        <OnboardingCardHeader
          icon={<FolderIcon className="size-8" />}
          subtitle="Choose where your agents will work."
          title="Create a new project"
        />

        <CardContent>
          <div className="flex flex-col gap-6">
            <div className="flex flex-col gap-2">
              <Label htmlFor="name">Project Name</Label>
              <Input
                required
                id="name"
                placeholder="Soil Mining"
                type="text"
                value={viewmodel.project.name}
                onChange={(e) => handleNameChange(e.target.value)}
              />
            </div>
            <LabeledTextarea
              label="Description"
              placeholder="What this project is for, who it serves, and what the team is building."
              value={viewmodel.project.description}
              onChange={handleDescriptionChange}
            />
            <div className="flex flex-col gap-2">
              <div className="flex flex-col gap-2">
                <Label htmlFor="working-directories">Folders</Label>

                {viewmodel.project.workingDirectories.map((directory, index) => (
                  <InputGroup key={index}>
                    <InputGroupInput
                      id={`working-directory-${index}`}
                      placeholder="/Users/[USERNAME]/projects/soil-mining"
                      type="text"
                      value={directory}
                      onChange={(e) => handleWorkingDirectoryChange(index, e.target.value)}
                    />
                    <InputGroupAddon align="inline-end">
                      <InputGroupButton
                        size="xs"
                        variant="destructive-ghost"
                        onClick={() => handleRemoveWorkingDirectory(index)}
                      >
                        <TrashIcon className="size-4" />
                      </InputGroupButton>
                    </InputGroupAddon>
                  </InputGroup>
                ))}
                <Button type="button" variant="outline" onClick={handleAddWorkingDirectory}>
                  Add Folder
                </Button>
              </div>
            </div>
          </div>
        </CardContent>
        <CardFooter className="flex justify-between">
          <Button variant="ghost" onClick={() => viewmodel.setStep(OnboardingStep.Provider)}>
            <ArrowLeftIcon className="size-4" /> Back
          </Button>

          <Button
            disabled={viewmodel.project.id ? viewmodel.project.isDirty : !viewmodel.project.isValid}
            type="submit"
          >
            <ArrowRightIcon className="size-4" /> {verb}
          </Button>
        </CardFooter>
      </form>
    </Card>
  )
})
