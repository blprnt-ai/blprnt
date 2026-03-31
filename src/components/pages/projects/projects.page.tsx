import { Page } from '@/components/layouts/page'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { ProjectsDirectory } from './components/projects-directory'
import { useProjectsViewmodel } from './projects.viewmodel'

export const ProjectsPage = () => {
  const viewmodel = useProjectsViewmodel()

  return (
    <Page className="overflow-y-auto px-3 pb-6 md:px-5">
      <div className="mx-auto flex w-full max-w-7xl flex-col gap-4">
        <Card>
          <CardHeader>
            <CardTitle>Projects</CardTitle>
            <CardDescription>Browse project workspaces and open one to review or edit its working directories.</CardDescription>
          </CardHeader>
          <CardContent>
            <p className="text-sm text-muted-foreground">
              {viewmodel.projects.length} {viewmodel.projects.length === 1 ? 'project' : 'projects'}
            </p>
          </CardContent>
        </Card>

        <ProjectsDirectory />
      </div>
    </Page>
  )
}
