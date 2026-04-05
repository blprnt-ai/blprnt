import { ProjectDirectoriesCard } from './project-directories-card'
import { ProjectMetadataCard } from './project-metadata-card'
import { ProjectOverviewCard } from './project-overview-card'

export const ProjectOverviewTab = () => {
  return (
    <div className="grid gap-4 xl:grid-cols-[minmax(0,1.5fr)_360px]">
      <div className="grid min-w-0 gap-4">
        <ProjectOverviewCard />
        <ProjectDirectoriesCard />
      </div>
      <div className="min-w-0">
        <ProjectMetadataCard />
      </div>
    </div>
  )
}