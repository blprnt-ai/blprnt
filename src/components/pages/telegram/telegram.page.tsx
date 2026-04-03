import { Page } from '@/components/layouts/page'
import { TelegramComingSoon } from './telegram-coming-soon'

export const TelegramPage = () => {
  return (
    <Page className="overflow-y-auto px-3 pb-6 md:px-5">
      <div className="mx-auto w-full max-w-5xl">
        <TelegramComingSoon />
      </div>
    </Page>
  )
}