import { useNavigate } from '@tanstack/react-router'
import { FolderPlusIcon } from 'lucide-react'
import { useState } from 'react'
import { ProjectForm } from '@/components/forms/project'
import { ProjectFormViewmodel } from '@/components/forms/project/project-form.viewmodel'
import { Page } from '@/components/layouts/page'
import { Button } from '@/components/ui/button'
import { ProjectsDirectory } from './components/projects-directory'
import { useProjectsViewmodel } from './projects.viewmodel'

export const ProjectsPage = () => {
  const viewmodel = useProjectsViewmodel()
  const navigate = useNavigate()
  const [projectFormViewmodel] = useState(
    () =>
      new ProjectFormViewmodel(async (project) => {
        await navigate({
          params: { projectId: project.id },
          to: '/projects/$projectId',
        })
      }),
  )

  return (
    <Page className="overflow-y-auto px-3 pb-6 md:px-5">
      <div className="mx-auto flex w-full max-w-7xl flex-col gap-4">
        <div className="flex justify-end">
          <Button type="button" variant="secondary" onClick={projectFormViewmodel.open}>
            <FolderPlusIcon />
            Add project
          </Button>
        </div>

        <ProjectsDirectory />
        <ProjectForm viewmodel={projectFormViewmodel} />
      </div>
    </Page>
  )
}
