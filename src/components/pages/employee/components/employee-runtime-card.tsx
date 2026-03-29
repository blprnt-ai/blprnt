import { Sparkles } from 'lucide-react'
import type { Provider } from '@/bindings/Provider'
import { LabeledInput } from '@/components/molecules/labeled-input'
import { LabeledSelect } from '@/components/molecules/labeled-select'
import { LabeledSwitch } from '@/components/molecules/labeled-switch'
import { LabeledTextarea } from '@/components/molecules/labeled-textarea'
import { MarkdownEditor } from '@/components/organisms/markdown-editor'
import { SlugSelect } from '@/components/organisms/slug-select'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Label } from '@/components/ui/label'
import { useEmployeeViewmodel } from '../employee.viewmodel'
import { formatProvider, isSameProvider } from '../utils'

const providerOptions: { label: string; value: Provider }[] = [
  { label: 'Anthropic', value: 'anthropic' },
  { label: 'Claude Code', value: 'claude_code' },
  { label: 'Codex', value: 'codex' },
  { label: 'OpenAI', value: 'openai' },
  { label: 'OpenRouter', value: 'open_router' },
]

export const EmployeeRuntimeCard = () => {
  const viewmodel = useEmployeeViewmodel()
  const { employee } = viewmodel

  if (!employee) return null

  const handleProviderChange = (value: Provider | null) => {
    if (!value) return

    if (!isSameProvider(employee.provider, value)) viewmodel.setSlug('')

    viewmodel.setProvider(value)
  }

  return (
    <Card className="border-border/60">
      <CardHeader>
        <CardTitle>Runtime</CardTitle>
        <CardDescription>
          Define cadence, throughput, and the prompt used when this employee wakes itself up.
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-5">
        <div className="rounded-2xl border border-border/60 bg-background p-4 shadow-sm">
          <div className="space-y-2">
            <Label>Heartbeat prompt</Label>
            <p className="text-sm leading-6 text-muted-foreground">
              This is the standing instruction the agent uses when it wakes and reviews its work.
            </p>
            <MarkdownEditor
              editorClassName="min-h-[320px]"
              placeholder="Review open work, priorities, blockers, and next decisions..."
              value={employee.heartbeat_prompt}
              onChange={(value) => (employee.heartbeat_prompt = value)}
            />
          </div>
        </div>

        <div className="grid gap-4 md:grid-cols-2">
          <LabeledInput
            label="Heartbeat interval"
            value={employee.heartbeat_interval_sec.toString()}
            onChange={(value) => {
              const parsed = Number.parseInt(value, 10)
              if (!Number.isNaN(parsed)) employee.heartbeat_interval_sec = parsed
            }}
          />
          <LabeledInput
            label="Max concurrent runs"
            value={employee.max_concurrent_runs.toString()}
            onChange={(value) => {
              const parsed = Number.parseInt(value, 10)
              if (!Number.isNaN(parsed)) employee.max_concurrent_runs = parsed
            }}
          />
        </div>

        <LabeledSwitch
          hint="Turn this on if the coordinator should wake the employee only when work arrives."
          label="Wake on demand"
          value={employee.wake_on_demand}
          onChange={(value) => (employee.wake_on_demand = value)}
        />

        <div className="grid gap-4 md:grid-cols-2">
          <LabeledSelect
            label="Provider"
            options={providerOptions}
            selectedValue={formatProvider(employee.provider)}
            value={employee.provider}
            onChange={handleProviderChange}
          />
          <SlugSelect
            provider={employee.provider}
            slug={employee.slug}
            onChange={(value) => viewmodel.setSlug(value ?? '')}
          />
        </div>

        <div className="space-y-4">
          <LabeledTextarea
            hint="Separate capabilities with commas."
            label="Capability list"
            placeholder="planning, strategy, hiring"
            value={viewmodel.capabilitiesValue}
            onChange={viewmodel.setCapabilities}
          />

          <div className="rounded-2xl border border-border/60 bg-muted/20 p-4">
            <div className="mb-2 flex items-center gap-2 text-sm font-medium">
              <Sparkles className="size-4" />
              Writing guidance
            </div>
            <p className="text-sm leading-6 text-muted-foreground">
              Keep this list action-oriented. Short phrases like “roadmapping”, “budget approval”, or “performance
              review” scan better than long sentences.
            </p>
          </div>
        </div>
      </CardContent>
    </Card>
  )
}
