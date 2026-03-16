import { observer } from 'mobx-react-lite'
import { Button } from '@/components/atoms/button'
import { FormField } from './form-field'
import { useModelsV2ViewModel } from './models-v2.viewmodel'

export const CustomModelForm = observer(() => {
  const viewmodel = useModelsV2ViewModel()

  return (
    <div className="rounded-md border p-4 space-y-3">
      <div className="grid gap-3 md:grid-cols-2">
        <FormField
          label="Model ID"
          value={viewmodel.customModelDraft.id}
          onChange={(value) => viewmodel.setCustomModelDraftField('id', value)}
        />
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
          label="Provider slug"
          value={viewmodel.customModelDraft.providerSlug}
          onChange={(value) => viewmodel.setCustomModelDraftField('providerSlug', value)}
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