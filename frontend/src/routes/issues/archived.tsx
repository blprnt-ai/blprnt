import { createFileRoute } from '@tanstack/react-router'
import { ArchivedIssuesPage } from '@/components/pages/issues-archived'

export const Route = createFileRoute('/issues/archived')({
  component: ArchivedIssuesPage,
  staticData: {
    breadcrumb: 'Archived issues',
  },
})