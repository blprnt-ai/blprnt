import { Info } from 'lucide-react'
import { useEffect } from 'react'
import { AlertBox } from '@/components/atoms/alert-box'
import { Field } from '@/components/atoms/field'
import { InputGroup, InputGroupAddon } from '@/components/atoms/input-group'
import { Label } from '@/components/atoms/label'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/atoms/select'
import { TooltipMacro } from '@/components/atoms/tooltip-macro'
import { llmModelsModel } from '@/lib/models/llm-models.model'
import { newModelOverride } from '@/lib/utils/default-models'

interface ModelOverrideProps {
  modelOverride: string | undefined
  onSetModelOverride: (modelOverride: string) => void
}

export const ModelOverride = ({ modelOverride, onSetModelOverride }: ModelOverrideProps) => {
  // biome-ignore lint/correctness/useExhaustiveDependencies: we only want to run this once
  useEffect(() => {
    if (modelOverride === newModelOverride) {
      onSetModelOverride(llmModelsModel.models[0]?.slug ?? newModelOverride)
    }
  }, [])

  if (llmModelsModel.models.length === 0)
    return <AlertBox description="No models are enabled. Please check your app settings." variant="danger" />

  return (
    <Field>
      <InputGroup className="justify-between" data-tour="session-model-select">
        <InputGroupAddon className="w-44 justify-start">
          <Label htmlFor="model_override">Model</Label>

          <TooltipMacro
            tooltip={
              <div>
                <div>
                  <div className="font-semibold">Override the default model for the session.</div>
                  <div>This will be used for all requests made in this session.</div>
                  <div className="font-medium">You can toggle models on and off in the settings page.</div>
                </div>
              </div>
            }
          >
            <Info className="h-4 w-4 text-primary cursor-help" />
          </TooltipMacro>
        </InputGroupAddon>
        <InputGroupAddon>
          <Select value={modelOverride ?? ''} onValueChange={(value) => onSetModelOverride(value)}>
            <SelectTrigger className="border-0 pr-0" size="sm">
              <SelectValue placeholder="Select a model">
                {llmModelsModel.models.find((m) => m.slug === modelOverride)?.name || 'Select a model'}
              </SelectValue>
            </SelectTrigger>
            <SelectContent>
              {llmModelsModel.modelOptions.map((m) => (
                <SelectItem key={m.model} className="w-full" useItemText={false} value={m.model}>
                  <div className="flex justify-between w-full">
                    <div>{m.display_name}</div>
                  </div>
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </InputGroupAddon>
      </InputGroup>
    </Field>
  )
}
