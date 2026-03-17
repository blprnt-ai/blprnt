import { createFileRoute } from '@tanstack/react-router'
import { ProjectPage } from '@/components/pages/project'

export const Route = createFileRoute('/projects/$projectId/')({
  component: ProjectPage,
})
