import { observer } from 'mobx-react-lite'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { useTelegramViewmodel } from '../telegram.viewmodel'

export const TelegramLinkedChats = observer(() => {
  const viewmodel = useTelegramViewmodel()

  return (
    <Card>
      <CardHeader>
        <CardTitle>Linked chats</CardTitle>
        <CardDescription>Current Telegram connections for this workspace.</CardDescription>
      </CardHeader>
      <CardContent>
        {viewmodel.links.length === 0 ? (
          <div className="space-y-2 text-sm text-muted-foreground">
            <p>No chats linked yet.</p>
            <p>Generate a link code, then send <code>{viewmodel.linkCommand}</code> to the bot.</p>
          </div>
        ) : null}
        <div className="space-y-3">
          {viewmodel.links.map((link) => (
            <div key={link.id} className="rounded-sm border border-border/80 bg-muted/20 p-4 text-sm">
              <div className="flex flex-wrap items-center justify-between gap-2">
                <div className="font-medium">Chat {link.telegram_chat_id.toString()}</div>
                <div className="text-muted-foreground">{link.status}</div>
              </div>
              <div className="mt-2 space-y-1 text-muted-foreground">
                <p>Linked {formatDateTime(link.created_at)}</p>
                {link.last_seen_at ? <p>Last seen {formatDateTime(link.last_seen_at)}</p> : null}
              </div>
            </div>
          ))}
        </div>

        <div className="mt-4 flex flex-wrap gap-2 text-sm text-muted-foreground">
          <span>After linking, use:</span>
          <code>/issue</code>
          <code>/comment</code>
          <code>/watch</code>
          <code>/run</code>
        </div>
      </CardContent>
    </Card>
  )
})

const formatDateTime = (value: string) => {
  return new Intl.DateTimeFormat('en', {
    dateStyle: 'medium',
    timeStyle: 'short',
  }).format(new Date(value))
}