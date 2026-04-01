import { createFileRoute } from '@tanstack/react-router'
import { RunDraftProvider } from '@/components/pages/run/run-draft.provider'

export const Route = createFileRoute('/employees/$employeeId/chat')({
  component: RunDraftProvider,
  staticData: {
    breadcrumb: 'Chat',
  },
})
