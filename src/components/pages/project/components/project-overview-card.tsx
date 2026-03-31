import { LabeledInput } from '@/components/molecules/labeled-input'
import { LabeledTextarea } from '@/components/molecules/labeled-textarea'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { useProjectViewmodel } from '../project.viewmodel'

export const ProjectOverviewCard = () => {
  const viewmodel = useProjectViewmodel()
  const { project } = viewmodel

  if (!project) return null

  return (
    <Card className="border-border/60">
      <CardHeader>
        <CardTitle>Project</CardTitle>
        <CardDescription>Set the name and description used throughout the workspace and runtime context.</CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        <LabeledInput
          label="Project name"
          placeholder="Customer portal"
          value={project.name}
          onChange={(value) => (project.name = value)}
        />
        <LabeledTextarea
          label="Description"
          placeholder="What this project is for, who it serves, and what the team is building."
          value={project.description}
          onChange={(value) => (project.description = value)}
        />

        {viewmodel.errorMessage && <p className="text-sm text-destructive">{viewmodel.errorMessage}</p>}
      </CardContent>
    </Card>
  )
}
