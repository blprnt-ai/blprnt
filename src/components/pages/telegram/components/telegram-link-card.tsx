import { RefreshCwIcon, SparklesIcon } from 'lucide-react'
import { observer } from 'mobx-react-lite'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { useTelegramViewmodel } from '../telegram.viewmodel'

export const TelegramLinkCard = observer(() => {
  const viewmodel = useTelegramViewmodel()

  return (
    <Card>
      <CardHeader>
        <CardTitle>Link your chat</CardTitle>
        <CardDescription>Generate a code, then send it to the bot from Telegram.</CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        <div className="flex flex-wrap items-center gap-3">
          <Button disabled={!viewmodel.canGenerateLinkCode} type="button" onClick={() => void viewmodel.generateLinkCode()}>
            <SparklesIcon />
            {viewmodel.isGeneratingCode ? 'Generating...' : 'Generate link code'}
          </Button>
          <Button type="button" variant="outline" onClick={() => void viewmodel.refreshLinks()}>
            <RefreshCwIcon />
            Refresh links
          </Button>
        </div>

        {!viewmodel.isReadyToLink ? <p className="text-sm text-muted-foreground">Save bot settings first.</p> : null}

        <ol className="space-y-2 text-sm text-muted-foreground">
          <li>1. Open Telegram and find {viewmodel.botHandle ? <strong>@{viewmodel.botHandle}</strong> : <strong>your bot</strong>}.</li>
          <li>2. Send <code>{viewmodel.linkCommand}</code>.</li>
        </ol>
      </CardContent>
    </Card>
  )
})