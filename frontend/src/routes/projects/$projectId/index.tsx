import { createFileRoute } from '@tanstack/react-router'
import { ProjectPage } from '@/components/pages/project'
import { AppModel } from '@/models/app.model'

export const Route = createFileRoute('/projects/$projectId/')({
  component: ProjectPage,
  staticData: {
    breadcrumb: ({ projectId }: Record<string, string>) =>
      AppModel.instance.resolveProjectName(projectId) ?? `Project ${projectId.slice(0, 8)}`,
  },
})
