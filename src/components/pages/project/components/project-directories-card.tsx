import { Plus, TrashIcon } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { InputGroup, InputGroupAddon, InputGroupButton, InputGroupInput } from '@/components/ui/input-group'
import { useProjectViewmodel } from '../project.viewmodel'

export const ProjectDirectoriesCard = () => {
  const viewmodel = useProjectViewmodel()
  const { project } = viewmodel

  if (!project) return null

  return (
    <Card className="border-border/60">
      <CardHeader>
        <CardTitle>Working Directories</CardTitle>
        <CardDescription>
          Add every local directory this project can operate in. These paths are used by agents and tools.
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        <div className="space-y-3">
          {project.workingDirectories.length > 0 ? (
            project.workingDirectories.map((directory, index) => (
              <InputGroup key={`${directory}-${index}`}>
                <InputGroupInput
                  placeholder="/Users/[USERNAME]/projects/example"
                  type="text"
                  value={directory}
                  onChange={(event) => project.setWorkingDirectory(index, event.target.value)}
                />
                <InputGroupAddon align="inline-end">
                  <InputGroupButton
                    size="xs"
                    variant="destructive-ghost"
                    onClick={() => project.removeWorkingDirectory(index)}
                  >
                    <TrashIcon className="size-4" />
                  </InputGroupButton>
                </InputGroupAddon>
              </InputGroup>
            ))
          ) : (
            <div className="rounded-2xl border border-dashed border-border/70 px-4 py-6 text-sm text-muted-foreground">
              Add at least one working directory so agents know where this project lives.
            </div>
          )}
        </div>

        <div className="flex flex-wrap items-center justify-between gap-3">
          <p className="text-sm text-muted-foreground font-light">Use absolute paths.</p>
          <Button type="button" variant="outline" onClick={project.addWorkingDirectory}>
            <Plus className="size-4" />
            Add folder
          </Button>
        </div>
      </CardContent>
    </Card>
  )
}
