import { createFileRoute } from '@tanstack/react-router'
import { RunProvider } from '@/components/pages/run/run.provider'

export const Route = createFileRoute('/runs/$runId/')({
  component: RunProvider,
  staticData: {
    breadcrumb: ({ runId }: Record<string, string>) => `Run ${runId.slice(0, 8)}`,
  },
})
