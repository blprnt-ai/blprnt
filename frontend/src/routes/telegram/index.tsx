import { createFileRoute } from '@tanstack/react-router'
import { TelegramPage } from '@/components/pages/telegram'

export const Route = createFileRoute('/telegram/')({
  component: TelegramPage,
  staticData: {
    breadcrumb: 'Telegram',
  },
})