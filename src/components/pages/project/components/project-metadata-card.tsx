import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { MetadataRow } from '@/components/pages/issue/components/metadata-row'
import { useProjectViewmodel } from '../project.viewmodel'
import { formatDate, formatDirectoryCount } from '../utils'

export const ProjectMetadataCard = () => {
  const viewmodel = useProjectViewmodel()
  const { project } = viewmodel

  if (!project) return null

  return (
    <Card>
      <CardHeader>
        <CardTitle>Metadata</CardTitle>
      </CardHeader>
      <CardContent className="space-y-4">
        <MetadataRow label="Project ID" value={project.id} />
        <MetadataRow label="Folders" value={formatDirectoryCount(viewmodel.workingDirectoryCount)} />
        <MetadataRow label="Created" value={formatDate(project.createdAt)} />
        <MetadataRow label="Last updated" value={formatDate(project.updatedAt)} />
      </CardContent>
    </Card>
  )
}
