import { createFileRoute } from '@tanstack/react-router'
import { DashboardProvider } from '@/components/pages/dashboard/dashboard.provider'

export const Route = createFileRoute('/')({
  component: DashboardProvider,
  staticData: {
    breadcrumb: 'Dashboard',
  },
})
