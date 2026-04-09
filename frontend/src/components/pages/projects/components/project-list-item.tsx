import { Link } from '@tanstack/react-router'
import type { ProjectDto } from '@/bindings/ProjectDto'
import { formatDate, formatDirectoryCount } from '@/components/pages/project/utils'
import { Card, CardContent } from '@/components/ui/card'

interface ProjectListItemProps {
  project: ProjectDto
}

export const ProjectListItem = ({ project }: ProjectListItemProps) => {
  return (
    <Link params={{ projectId: project.id }} to="/projects/$projectId">
      <Card className="transition-colors hover:bg-muted/40">
        <CardContent className="flex flex-col gap-3">
          <div className="space-y-1">
            <div className="font-medium">{project.name}</div>
            <div className="text-sm text-muted-foreground">
              {formatDirectoryCount(project.working_directories.length)}
            </div>
            {project.description ? (
              <p className="line-clamp-1 text-sm text-muted-foreground">{project.description}</p>
            ) : null}
          </div>
          <div className="space-y-1 text-sm text-muted-foreground">
            <p>{project.working_directories[0] ?? 'No working directory'}</p>
            <p>Updated {formatDate(new Date(project.updated_at))}</p>
          </div>
        </CardContent>
      </Card>
    </Link>
  )
}
