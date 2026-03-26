import { createFileRoute } from '@tanstack/react-router'
import { IssuesPage } from '@/components/pages/issues'

export const Route = createFileRoute('/issues/')({
  component: IssuesPage,
})
