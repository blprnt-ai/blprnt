import { SaveIcon, SendIcon } from 'lucide-react'
import { observer } from 'mobx-react-lite'
import { LabeledSwitch } from '@/components/molecules/labeled-switch'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { useTelegramViewmodel } from '../telegram.viewmodel'

export const TelegramConfigForm = observer(() => {
  const viewmodel = useTelegramViewmodel()

  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          <SendIcon className="size-4" />
          Telegram
        </CardTitle>
        <CardDescription>Configure the shared bot once, then link your Telegram chat.</CardDescription>
      </CardHeader>
      <CardContent className="space-y-5">
        <LabeledSwitch inline label="Enabled" value={viewmodel.enabled} onChange={viewmodel.setEnabled} />

        <div className="grid gap-4 md:grid-cols-2">
          <Field label="Bot token">
            <Input
              placeholder="123456:ABCDEF..."
              type="password"
              value={viewmodel.botToken}
              onChange={(event) => viewmodel.setBotToken(event.target.value)}
            />
          </Field>
          <Field label="Bot username">
            <Input
              placeholder="blprnt_bot"
              value={viewmodel.botUsername}
              onChange={(event) => viewmodel.setBotUsername(event.target.value)}
            />
          </Field>
          <Field label="Delivery mode">
            <Select value={viewmodel.deliveryMode} onValueChange={(value) => viewmodel.setDeliveryMode(value as 'webhook' | 'polling')}>
              <SelectTrigger className="w-full">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="webhook">webhook</SelectItem>
                <SelectItem value="polling">polling</SelectItem>
              </SelectContent>
            </Select>
          </Field>
          <Field label="Parse mode">
            <Select
              value={viewmodel.parseMode ?? 'none'}
              onValueChange={(value) => viewmodel.setParseMode(value === 'none' ? null : (value as 'html' | 'markdown_v2'))}
            >
              <SelectTrigger className="w-full">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="none">none</SelectItem>
                <SelectItem value="html">html</SelectItem>
                <SelectItem value="markdown_v2">markdown_v2</SelectItem>
              </SelectContent>
            </Select>
          </Field>
          <Field className="md:col-span-2" label="Webhook URL">
            <Input
              placeholder="https://your-host/api/v1/integrations/telegram/webhook"
              value={viewmodel.webhookUrl}
              onChange={(event) => viewmodel.setWebhookUrl(event.target.value)}
            />
          </Field>
          <Field className="md:col-span-2" label="Webhook secret">
            <Input
              placeholder="shared webhook secret"
              type="password"
              value={viewmodel.webhookSecret}
              onChange={(event) => viewmodel.setWebhookSecret(event.target.value)}
            />
          </Field>
        </div>

        <div className="flex flex-wrap items-center gap-3">
          <Button disabled={viewmodel.isSaving} type="button" onClick={() => void viewmodel.saveConfig()}>
            <SaveIcon />
            {viewmodel.isSaving ? 'Saving...' : 'Save settings'}
          </Button>
          {viewmodel.saveMessage ? <p className="text-sm text-muted-foreground">{viewmodel.saveMessage}</p> : null}
        </div>
      </CardContent>
    </Card>
  )
})

interface FieldProps {
  children: React.ReactNode
  className?: string
  label: string
}

const Field = ({ children, className, label }: FieldProps) => {
  return (
    <div className={className}>
      <Label className="mb-2">{label}</Label>
      {children}
    </div>
  )
}