import { FolderIcon } from 'lucide-react'
import { LabeledInput } from '@/components/molecules/labeled-input'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { MetadataRow } from '@/components/pages/issue/components/metadata-row'
import { useProjectViewmodel } from '../project.viewmodel'
import { formatDirectoryCount } from '../utils'

export const ProjectOverviewCard = () => {
  const viewmodel = useProjectViewmodel()
  const { project } = viewmodel

  if (!project) return null

  return (
    <Card>
      <CardHeader>
        <CardTitle>Overview</CardTitle>
      </CardHeader>
      <CardContent className="space-y-4">
        <div className="flex items-center gap-2 text-muted-foreground">
          <FolderIcon className="size-4" />
          <span className="text-sm">{formatDirectoryCount(project.workingDirectories.length)}</span>
        </div>

        {viewmodel.isEditing ? (
          <LabeledInput label="Project name" value={project.name} onChange={(value) => (project.name = value)} />
        ) : (
          <MetadataRow label="Project name" value={project.name || 'Untitled project'} />
        )}
      </CardContent>
    </Card>
  )
}
