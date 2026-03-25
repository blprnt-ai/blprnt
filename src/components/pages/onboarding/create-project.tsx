import { TrashIcon } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from '@/components/ui/card'
import { Input } from '@/components/ui/input'
import { InputGroup, InputGroupAddon, InputGroupButton, InputGroupInput } from '@/components/ui/input-group'
import { Label } from '@/components/ui/label'
import { useOnboardingViewmodel } from './onboarding.viewmodel'

export const CreateProject = () => {
  const viewmodel = useOnboardingViewmodel()

  const handleNameChange = (value: string) => {
    viewmodel.project.name = value
  }

  const handleAddWorkingDirectory = () => {
    viewmodel.project.workingDirectories.push('')
  }

  const handleRemoveWorkingDirectory = (index: number) => {
    viewmodel.project.workingDirectories.splice(index, 1)
  }

  const handleWorkingDirectoryChange = (index: number, value: string) => {
    viewmodel.project.workingDirectories[index] = value
  }

  return (
    <Card className="w-full max-w-lg">
      <CardHeader>
        <CardTitle>Create a new project</CardTitle>
        <CardDescription>Enter the name of your project and add at least one folder</CardDescription>
      </CardHeader>
      <CardContent>
        <form>
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
        </form>
      </CardContent>
      <CardFooter className="flex justify-end">
        <Button disabled={!viewmodel.project.isValid} type="submit" onClick={() => viewmodel.saveProject()}>
          Create Project
        </Button>
      </CardFooter>
    </Card>
  )
}
