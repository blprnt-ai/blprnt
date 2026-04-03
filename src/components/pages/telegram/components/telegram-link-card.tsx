import { RefreshCwIcon, SparklesIcon } from 'lucide-react'
import { observer } from 'mobx-react-lite'
import { CopyButton } from '@/components/ui/copy-button'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { useTelegramViewmodel } from '../telegram.viewmodel'

export const TelegramLinkCard = observer(() => {
  const viewmodel = useTelegramViewmodel()

  return (
    <Card>
      <CardHeader>
        <CardTitle>Link your chat</CardTitle>
        <CardDescription>Generate a short-lived code, send it to the bot, then use Telegram commands from that chat.</CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        <div className="flex flex-wrap items-center gap-3">
          <Button disabled={viewmodel.isGeneratingCode} type="button" onClick={() => void viewmodel.generateLinkCode()}>
            <SparklesIcon />
            {viewmodel.isGeneratingCode ? 'Generating...' : 'Generate link code'}
          </Button>
          <Button type="button" variant="outline" onClick={() => void viewmodel.refreshLinks()}>
            <RefreshCwIcon />
            Refresh links
          </Button>
        </div>

        {viewmodel.latestLinkCode ? (
          <div className="rounded-sm border border-border/80 bg-muted/25 p-4">
            <div className="flex flex-wrap items-center gap-3">
              <code className="text-lg font-semibold tracking-[0.18em]">{viewmodel.latestLinkCode.code}</code>
              <CopyButton content={viewmodel.latestLinkCode.code} variant="outline" />
            </div>
            <p className="mt-2 text-sm text-muted-foreground">Expires {formatDateTime(viewmodel.latestLinkCode.record.expires_at)}</p>
          </div>
        ) : null}

        <ol className="space-y-2 text-sm text-muted-foreground">
          <li>1. Open Telegram and find {viewmodel.botHandle ? <strong>@{viewmodel.botHandle}</strong> : <strong>your bot</strong>}.</li>
          <li>
            2. Send <code>{viewmodel.linkCommand}</code>
            {viewmodel.latestLinkCode ? <CopyButton className="ml-2 inline-flex align-middle" content={viewmodel.linkCommand} size="xs" variant="link" /> : null}
          </li>
          <li>3. After linking, use <code>/issue</code>, <code>/comment</code>, <code>/watch</code>, and <code>/run</code>.</li>
        </ol>
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