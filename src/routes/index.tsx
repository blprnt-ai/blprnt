import { createFileRoute } from '@tanstack/react-router'
import { DashboardPage } from '@/components/pages/dashboard/dashboard.page'

export const Route = createFileRoute('/')({
  component: DashboardPage,
  staticData: {
    breadcrumb: 'Dashboard',
  },
})
