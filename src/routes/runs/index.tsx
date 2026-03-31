import { createFileRoute } from '@tanstack/react-router'
import { RunsProvider } from '@/components/pages/runs/runs.provider'

export const Route = createFileRoute('/runs/')({
  component: RunsProvider,
  staticData: {
    breadcrumb: 'Runs',
  },
})
