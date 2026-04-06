import { MessageCircleIcon } from 'lucide-react'
import { observer } from 'mobx-react-lite'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { CopyButton } from '@/components/ui/copy-button'
import { useTelegramViewmodel } from '../telegram.viewmodel'

export const TelegramOverviewCard = observer(() => {
  const viewmodel = useTelegramViewmodel()

  return (
    <Card>
      <CardHeader className="gap-4 md:flex-row md:items-start md:justify-between">
        <div className="space-y-1">
          <CardTitle className="flex items-center gap-2">
            <MessageCircleIcon className="size-4" />
            Telegram
          </CardTitle>
          <CardDescription>{viewmodel.summaryText}</CardDescription>
        </div>

        <div className={`inline-flex items-center rounded-full px-3 py-1 text-xs font-medium ${viewmodel.statusClassName}`}>
          {viewmodel.statusLabel}
        </div>
      </CardHeader>

      <CardContent className="space-y-4">
        <div className="flex flex-wrap gap-2 text-xs text-muted-foreground">
          <StatusChip label={viewmodel.hasSavedConfig ? 'Bot configured' : 'Bot not configured'} />
          <StatusChip label={viewmodel.enabled ? 'Enabled' : 'Disabled'} />
          <StatusChip label={viewmodel.linkedChatsLabel} />
        </div>
        {viewmodel.latestLinkCode ? (
          <div className="space-y-3 rounded-sm border border-border/80 bg-muted/25 p-4">
            <div className="flex flex-wrap items-center gap-3">
              <code className="text-lg font-semibold tracking-[0.18em]">{viewmodel.latestLinkCode.code}</code>
              <CopyButton content={viewmodel.latestLinkCode.code} variant="outline" />
            </div>

            <div className="flex flex-wrap items-center gap-2 text-sm text-muted-foreground">
              <span>Send</span>
              <code>{viewmodel.linkCommand}</code>
              <CopyButton content={viewmodel.linkCommand} size="xs" variant="link" />
            </div>

            <p className="text-sm text-muted-foreground">Expires {formatDateTime(viewmodel.latestLinkCode.record.expires_at)}</p>
          </div>
        ) : null}
      </CardContent>
    </Card>
  )
})

interface StatusChipProps {
  label: string
}

const StatusChip = ({ label }: StatusChipProps) => {
  return <span className="rounded-full border border-border/70 px-2.5 py-1">{label}</span>
}

const formatDateTime = (value: string) => {
  return new Intl.DateTimeFormat('en', {
    dateStyle: 'medium',
    timeStyle: 'short',
  }).format(new Date(value))
}