import { Page } from '@/components/layouts/page'
import { ProjectDirectoriesCard } from './components/project-directories-card'
import { ProjectHeader } from './components/project-header'
import { ProjectMetadataCard } from './components/project-metadata-card'
import { ProjectNotFound } from './components/project-not-found'
import { ProjectOverviewCard } from './components/project-overview-card'
import { useProjectViewmodel } from './project.viewmodel'

export const ProjectPage = () => {
  const viewmodel = useProjectViewmodel()

  if (!viewmodel.project) return <ProjectNotFound />

  return (
    <Page className="overflow-y-auto px-3 pb-6 md:px-5">
      <div className="mx-auto flex w-full max-w-7xl flex-col gap-4">
        <ProjectHeader />
        <div className="grid gap-4 xl:grid-cols-[minmax(0,1.5fr)_360px]">
          <div className="grid min-w-0 gap-4">
            <ProjectOverviewCard />
            <ProjectDirectoriesCard />
          </div>
          <div className="min-w-0">
            <ProjectMetadataCard />
          </div>
        </div>
      </div>
    </Page>
  )
}
