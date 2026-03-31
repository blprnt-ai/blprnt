import { PlusIcon, TrashIcon } from 'lucide-react'
import { Input } from '@/components/ui/input'
import { InputGroup, InputGroupAddon, InputGroupButton, InputGroupInput } from '@/components/ui/input-group'
import { Label } from '@/components/ui/label'
import { cn } from '@/lib/utils'
import type { ProjectFormViewmodel } from './project-form.viewmodel'

interface ProjectFormFieldsProps {
  viewmodel: ProjectFormViewmodel
}

export const ProjectFormFields = ({ viewmodel }: ProjectFormFieldsProps) => {
  const { project } = viewmodel

  return (
    <div className="flex min-h-0 flex-1 flex-col gap-6 overflow-y-auto px-4 py-2">
      <div className="flex flex-col gap-2">
        <Label htmlFor="project-name">Project name</Label>
        <Input
          required
          id="project-name"
          placeholder="Soil Mining"
          type="text"
          value={project.name}
          onChange={(event) => {
            project.name = event.target.value
          }}
        />
      </div>

      <div className="flex flex-col gap-3">
        <div className="flex items-center justify-between gap-3">
          <Label>Working directories</Label>
          <InputGroupButton size="sm" type="button" variant="outline" onClick={project.addWorkingDirectory}>
            <PlusIcon />
            Add folder
          </InputGroupButton>
        </div>

        <div className="flex flex-col gap-2">
          {project.workingDirectories.map((directory, index) => (
            <InputGroup key={`${index}-${directory}`}>
              <InputGroupInput
                id={`working-directory-${index}`}
                placeholder="/Users/[USERNAME]/projects/soil-mining"
                type="text"
                value={directory}
                onChange={(event) => project.setWorkingDirectory(index, event.target.value)}
              />
              <InputGroupAddon align="inline-end">
                <InputGroupButton
                  disabled={project.workingDirectories.length === 1}
                  size="xs"
                  type="button"
                  variant="destructive-ghost"
                  onClick={() => project.removeWorkingDirectory(index)}
                >
                  <TrashIcon className="size-4" />
                </InputGroupButton>
              </InputGroupAddon>
            </InputGroup>
          ))}

          <p className={cn('text-sm text-muted-foreground', project.isValid && 'sr-only')}>Add at least one folder.</p>
        </div>
      </div>
    </div>
  )
}
