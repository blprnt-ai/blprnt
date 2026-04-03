import { createFileRoute } from '@tanstack/react-router'
import { ProvidersPage } from '@/components/pages/providers'

export const Route = createFileRoute('/providers/')({
  component: ProvidersPage,
  staticData: {
    breadcrumb: 'Providers',
  },
})
