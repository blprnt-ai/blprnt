import { EmptyState } from '@/components/pages/issue/components/empty-state'
import { useProjectsViewmodel } from '../projects.viewmodel'
import { ProjectListItem } from './project-list-item'

export const ProjectsDirectory = () => {
  const viewmodel = useProjectsViewmodel()

  if (viewmodel.projects.length === 0) {
    return (
      <EmptyState
        description="Projects will appear here once they are created in your workspace."
        title="No projects yet"
      />
    )
  }

  return (
    <div className="grid gap-3 md:grid-cols-2 xl:grid-cols-3">
      {viewmodel.projects.map((project) => (
        <ProjectListItem key={project.id} project={project} />
      ))}
    </div>
  )
}
