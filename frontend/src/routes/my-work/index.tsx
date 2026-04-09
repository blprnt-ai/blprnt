import { createFileRoute } from '@tanstack/react-router'
import { MyWorkPage } from '@/components/pages/my-work'

export const Route = createFileRoute('/my-work/')({
  component: MyWorkPage,
  staticData: {
    breadcrumb: 'My Work',
  },
})
