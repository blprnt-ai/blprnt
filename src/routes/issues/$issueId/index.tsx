import { createFileRoute } from '@tanstack/react-router'
import { IssuePage } from '@/components/pages/issue'

export const Route = createFileRoute('/issues/$issueId/')({
  component: IssuePage,
  staticData: {
    breadcrumb: ({ issueId }: Record<string, string>) => `Issue ${issueId.slice(0, 8)}`,
  },
})
