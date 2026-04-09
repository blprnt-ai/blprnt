import { observer } from 'mobx-react-lite'
import { LabeledInput } from '@/components/molecules/labeled-input'
import { LabeledSwitch } from '@/components/molecules/labeled-switch'
import { LabeledTextarea } from '@/components/molecules/labeled-textarea'
import { Card, CardContent } from '@/components/ui/card'
import type { MinionModel } from '@/models/minion.model'

interface MinionFieldsProps {
  minion: MinionModel
}

export const MinionFields = observer(({ minion }: MinionFieldsProps) => {
  const isReadOnly = minion.isReadOnly

  return (
    <div className="flex flex-col gap-4">
      <LabeledInput disabled={isReadOnly} label="Name" placeholder="Research assistant" value={minion.displayName} onChange={(value) => (minion.displayName = value)} />
      <LabeledInput disabled={isReadOnly} label="Slug" placeholder="research-assistant" value={minion.slug} onChange={(value) => (minion.slug = value)} />
      <LabeledTextarea
        disabled={isReadOnly}
        label="Description"
        placeholder="What this minion is responsible for."
        value={minion.description}
        onChange={(value) => (minion.description = value)}
      />
      <LabeledSwitch disabled={isReadOnly} label="Enabled" value={minion.enabled} onChange={(value) => (minion.enabled = value)} />
      {minion.isSystem ? (
        <Card className="border-border/60 py-0">
          <CardContent className="px-5 py-4 text-sm text-muted-foreground">
            System minions are built in and read-only.
          </CardContent>
        </Card>
      ) : (
        <LabeledTextarea
          disabled={isReadOnly}
          label="Prompt"
          placeholder="Keep the minion prompt concise and task-specific."
          value={minion.prompt}
          onChange={(value) => (minion.prompt = value)}
        />
      )}
    </div>
  )
})