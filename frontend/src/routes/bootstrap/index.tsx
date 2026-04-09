import { createFileRoute } from '@tanstack/react-router'
import { BootstrapOwnerPage } from '@/components/pages/auth/bootstrap-owner.page'

export const Route = createFileRoute('/bootstrap/')({
  component: BootstrapOwnerPage,
})
