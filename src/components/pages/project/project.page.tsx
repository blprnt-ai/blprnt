import { Page } from '@/components/layouts/page'
import { ProjectDirectoriesCard } from './components/project-directories-card'
import { ProjectDirectoryActions } from './components/project-directory-actions'
import { ProjectHeader } from './components/project-header'
import { ProjectMetadataCard } from './components/project-metadata-card'
import { ProjectNotFound } from './components/project-not-found'
import { ProjectOverviewCard } from './components/project-overview-card'
import { useProjectViewmodel } from './project.viewmodel'

export const ProjectPage = () => {
  const viewmodel = useProjectViewmodel()

  if (!viewmodel.project) return <ProjectNotFound />

  return (
    <Page className="overflow-y-auto p-1 pr-2">
      <div className="flex flex-col gap-3">
        <ProjectHeader />
        <div className="flex flex-col gap-3 lg:flex-row lg:justify-between">
          <div className="flex min-w-0 flex-col gap-3">
            <ProjectOverviewCard />
            <ProjectDirectoriesCard />
            <ProjectDirectoryActions />
          </div>
          <div className="w-full lg:w-[320px]">
            <ProjectMetadataCard />
          </div>
        </div>
      </div>
    </Page>
  )
}
