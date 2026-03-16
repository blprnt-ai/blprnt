import { observer } from 'mobx-react-lite'
import { Button } from '@/components/atoms/button'
import { Switch } from '@/components/atoms/switch'
import { FormField } from './form-field'
import { useModelsV2ViewModel } from './models-v2.viewmodel'

export const CustomModelForm = observer(() => {
  const viewmodel = useModelsV2ViewModel()

  return (
    <div className="rounded-md border p-4 space-y-3">
      <div className="flex flex-col gap-3">
        <FormField
          helpText="The canonical slug of the model. e.g. openai/gpt-5.4"
          label="Model ID"
          value={viewmodel.customModelDraft.providerSlug}
          onChange={(value) => viewmodel.setCustomModelDraftField('providerSlug', value)}
        />
        <label className="flex flex-col gap-1.5">
          <span className="text-xs text-muted-foreground">Supports Reasoning</span>
          <Switch
            checked={viewmodel.customModelDraft.supportsReasoning}
            onCheckedChange={() => viewmodel.toggleCustomModelSupportsReasoning()}
          />
        </label>
        <FormField
          label="Display name"
          value={viewmodel.customModelDraft.name}
          onChange={(value) => viewmodel.setCustomModelDraftField('name', value)}
        />
        <FormField
          label="Context length"
          value={viewmodel.customModelDraft.contextLength}
          onChange={(value) => viewmodel.setCustomModelDraftField('contextLength', value)}
        />
        <FormField
          label="Price per million tokens"
          value={viewmodel.customModelDraft.promptPrice}
          onChange={(value) => viewmodel.setCustomModelDraftField('promptPrice', value)}
        />
      </div>
      {viewmodel.customModelFormError ? (
        <div className="text-sm text-destructive">{viewmodel.customModelFormError}</div>
      ) : null}
      <div className="flex gap-2">
        <Button size="sm" onClick={() => void viewmodel.saveCustomModel()}>
          Save custom model
        </Button>
        <Button size="sm" variant="ghost" onClick={viewmodel.closeCustomModelForm}>
          Cancel
        </Button>
      </div>
    </div>
  )
})
