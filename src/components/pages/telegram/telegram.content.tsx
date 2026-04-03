import { observer } from 'mobx-react-lite'
import { Card, CardContent } from '@/components/ui/card'
import { TelegramConfigForm } from './components/telegram-config-form'
import { TelegramLinkCard } from './components/telegram-link-card'
import { TelegramLinkedChats } from './components/telegram-linked-chats'
import { useTelegramViewmodel } from './telegram.viewmodel'

export const TelegramContent = observer(() => {
  const viewmodel = useTelegramViewmodel()

  return (
    <div className="flex flex-col gap-4">
      {viewmodel.errorMessage ? (
        <Card>
          <CardContent className="py-4 text-sm text-destructive">{viewmodel.errorMessage}</CardContent>
        </Card>
      ) : null}

      <TelegramConfigForm />
      <TelegramLinkCard />
      <TelegramLinkedChats />
    </div>
  )
})