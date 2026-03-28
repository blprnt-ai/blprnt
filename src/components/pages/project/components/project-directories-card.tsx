import { TrashIcon } from 'lucide-react'
import { EmptyState } from '@/components/pages/issue/components/empty-state'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { InputGroup, InputGroupAddon, InputGroupButton, InputGroupInput } from '@/components/ui/input-group'
import { useProjectViewmodel } from '../project.viewmodel'

export const ProjectDirectoriesCard = () => {
  const viewmodel = useProjectViewmodel()
  const { project } = viewmodel

  if (!project) return null

  return (
    <Card>
      <CardHeader>
        <CardTitle>Working directories</CardTitle>
      </CardHeader>
      <CardContent className="space-y-3">
        {project.workingDirectories.length > 0 ? (
          project.workingDirectories.map((directory, index) =>
            viewmodel.isEditing ? (
              <InputGroup key={`${directory}-${index}`}>
                <InputGroupInput
                  placeholder="/Users/[USERNAME]/projects/example"
                  type="text"
                  value={directory}
                  onChange={(event) => project.setWorkingDirectory(index, event.target.value)}
                />
                <InputGroupAddon align="inline-end">
                  <InputGroupButton size="xs" variant="destructive-ghost" onClick={() => project.removeWorkingDirectory(index)}>
                    <TrashIcon className="size-4" />
                  </InputGroupButton>
                </InputGroupAddon>
              </InputGroup>
            ) : (
              <div key={`${directory}-${index}`} className="rounded-sm border border-border/70 px-3 py-2 text-sm">
                {directory}
              </div>
            ),
          )
        ) : (
          <EmptyState
            description="Add at least one working directory so agents know where this project lives."
            title="No working directories"
          />
        )}
      </CardContent>
    </Card>
  )
}
