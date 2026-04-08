import { BookOpenText, ScrollText, SquareStack } from 'lucide-react'
import { observer } from 'mobx-react-lite'
import { Page } from '@/components/layouts/page'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { ProjectHeader } from './components/project-header'
import { ProjectMemoryTab } from './components/project-memory-tab'
import { ProjectNotFound } from './components/project-not-found'
import { ProjectOverviewTab } from './components/project-overview-tab'
import { ProjectPlansTab } from './components/project-plans-tab'
import { useProjectViewmodel } from './project.viewmodel'

export const ProjectPage = observer(() => {
  const viewmodel = useProjectViewmodel()

  if (!viewmodel.project) return <ProjectNotFound />

  return (
    <Page className="overflow-y-auto px-3 pb-6 md:px-5">
      <div className="mx-auto flex w-full max-w-7xl flex-col gap-4">
        <ProjectHeader />

        <Tabs
          value={viewmodel.activeTab}
          onValueChange={(value) => viewmodel.setActiveTab(value as typeof viewmodel.activeTab)}
        >
          <TabsList variant="line">
            <TabsTrigger value="overview">
              <BookOpenText className="size-4" />
              Overview
            </TabsTrigger>
            <TabsTrigger value="memory">
              <ScrollText className="size-4" />
              Memory
            </TabsTrigger>
            <TabsTrigger value="plans">
              <SquareStack className="size-4" />
              Plans
            </TabsTrigger>
          </TabsList>

          <TabsContent className="mt-5" value="overview">
            <ProjectOverviewTab />
          </TabsContent>

          <TabsContent className="mt-5" value="memory">
            <ProjectMemoryTab />
          </TabsContent>

          <TabsContent className="mt-5" value="plans">
            <ProjectPlansTab />
          </TabsContent>
        </Tabs>
      </div>
    </Page>
  )
})
