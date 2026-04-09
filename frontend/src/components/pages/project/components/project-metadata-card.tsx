import { Clock3, Fingerprint, FolderTree, MoonStar } from 'lucide-react'
import { observer } from 'mobx-react-lite'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { useProjectViewmodel } from '../project.viewmodel'
import { formatDate, formatDirectoryCount } from '../utils'

export const ProjectMetadataCard = observer(() => {
  const viewmodel = useProjectViewmodel()
  const { project } = viewmodel

  if (!project) return null

  return (
    <Card className="border-border/60">
      <CardHeader>
        <CardTitle>Metadata</CardTitle>
        <CardDescription>Reference details for auditing and orientation.</CardDescription>
      </CardHeader>
      <CardContent className="space-y-3">
        <MetadataRow multiline icon={Fingerprint} label="Project ID" value={project.id} />
        <MetadataRow
          icon={FolderTree}
          label="Working directories"
          value={formatDirectoryCount(viewmodel.workingDirectoryCount)}
        />
        <MetadataRow icon={MoonStar} label="Dreaming" value={project.dreamingEnabled ? 'Enabled' : 'Disabled'} />
        <MetadataRow icon={Clock3} label="Created" value={formatDate(project.createdAt)} />
        <MetadataRow icon={Clock3} label="Last updated" value={formatDate(project.updatedAt)} />
      </CardContent>
    </Card>
  )
})

const MetadataRow = ({
  icon: Icon,
  label,
  value,
  multiline = false,
}: {
  icon: React.ComponentType<{ className?: string }>
  label: string
  value: string
  multiline?: boolean
}) => {
  return (
    <div className="rounded-2xl border border-border/60 bg-muted/20 p-4">
      <div className="flex items-start gap-3">
        <div className="flex size-9 items-center justify-center rounded-full bg-background text-muted-foreground">
          <Icon className="size-4" />
        </div>
        <div className="min-w-0">
          <p className="text-sm font-medium">{label}</p>
          <p className={multiline ? 'break-all text-sm text-muted-foreground' : 'text-sm text-muted-foreground'}>
            {value}
          </p>
        </div>
      </div>
    </div>
  )
}
