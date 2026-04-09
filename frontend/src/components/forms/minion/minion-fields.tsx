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
  return (
    <div className="flex flex-col gap-4">
      <LabeledInput
        disabled={minion.isDefinitionReadOnly}
        label="Name"
        placeholder="Research assistant"
        value={minion.displayName}
        onChange={(value) => (minion.displayName = value)}
      />
      <LabeledInput
        disabled={minion.isDefinitionReadOnly}
        label="Slug"
        placeholder="research-assistant"
        value={minion.slug}
        onChange={(value) => (minion.slug = value)}
      />
      <LabeledTextarea
        disabled={minion.isDefinitionReadOnly}
        label="Description"
        placeholder="What this minion is responsible for."
        value={minion.description}
        onChange={(value) => (minion.description = value)}
      />
      <LabeledSwitch
        disabled={minion.isToggleReadOnly}
        label="Enabled"
        value={minion.enabled}
        onChange={(value) => (minion.enabled = value)}
      />
      {minion.isSystem ? (
        <Card className="border-border/60 py-0">
          <CardContent className="px-5 py-4 text-sm text-muted-foreground">
            This system minion uses a built-in runtime definition. Only its enabled state can be changed here.
          </CardContent>
        </Card>
      ) : (
        <LabeledTextarea
          disabled={minion.isDefinitionReadOnly}
          label="Prompt"
          placeholder="Keep the minion prompt concise and task-specific."
          value={minion.prompt}
          onChange={(value) => (minion.prompt = value)}
        />
      )}
    </div>
  )
})
