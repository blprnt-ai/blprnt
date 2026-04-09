import { observer } from 'mobx-react-lite'
import type { Provider } from '@/bindings/Provider'
import type { ReasoningEffort } from '@/bindings/ReasoningEffort'
import { LabeledInput } from '@/components/molecules/labeled-input'
import { LabeledSelect } from '@/components/molecules/labeled-select'
import { LabeledSwitch } from '@/components/molecules/labeled-switch'
import { LabeledTextarea } from '@/components/molecules/labeled-textarea'
import { MarkdownEditor } from '@/components/organisms/markdown-editor'
import { SlugSelect } from '@/components/organisms/slug-select'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Label } from '@/components/ui/label'
import { DEFAULT_REASONING_OPTION, formatDefaultReasoningLabel, reasoningEffortOptions } from '@/lib/reasoning'
import type { EmployeeRole } from '@/bindings/EmployeeRole'
import { useEmployeeViewmodel } from '../employee.viewmodel'
import { formatProvider, isSameProvider } from '../utils'

export const EmployeeRuntimeCard = observer(() => {
  const viewmodel = useEmployeeViewmodel()
  const { employee } = viewmodel

  if (!employee) return null

  const handleProviderChange = (value: Provider | null) => {
    if (!value) return

    if (!isSameProvider(employee.provider, value)) viewmodel.setSlug('')

    viewmodel.setProvider(value)
  }

  const preventEmptyRunsHint = getPreventEmptyRunsHint(employee.role)

  return (
    <Card className="border-border/60 z-20">
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

        <div className="grid gap-4 md:grid-cols-3">
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
          <LabeledSelect
            hint="Used by default for new turns unless a run composer overrides it."
            label="Reasoning level"
            selectedValue={formatDefaultReasoningLabel(employee.reasoning_effort)}
            value={employee.reasoning_effort ?? DEFAULT_REASONING_OPTION}
            options={[
              { label: formatDefaultReasoningLabel(null), value: DEFAULT_REASONING_OPTION },
              ...reasoningEffortOptions,
            ]}
            onChange={(value) => {
              employee.reasoning_effort = value === DEFAULT_REASONING_OPTION ? null : (value as ReasoningEffort)
            }}
          />
        </div>

        <div className="space-y-4 rounded-2xl border border-border/60 bg-muted/20 p-4">
          <div className="space-y-1">
            <Label>Wakeup behavior</Label>
            <p className="text-sm leading-6 text-muted-foreground">
              Pause status is controlled separately. These settings only shape how the coordinator wakes this employee.
            </p>
          </div>

          <div className="grid grid-cols-1 md:grid-cols-3">
            <LabeledSwitch
              hint="Disables scheduled timer wakeups only. Manual starts, conversations, and other non-timer paths still work."
              label="Scheduled timer wakeups"
              value={employee.timer_wakeups_enabled}
              onChange={(value) => (employee.timer_wakeups_enabled = value)}
            />

            <LabeledSwitch
              hint="Turn this on if the coordinator should wake the employee only when work arrives."
              label="Wake on demand"
              value={employee.wake_on_demand}
              onChange={(value) => (employee.wake_on_demand = value)}
            />

            <LabeledSwitch
              hint={preventEmptyRunsHint}
              label="Prevent empty runs"
              value={employee.prevent_empty_runs}
              onChange={(value) => (employee.prevent_empty_runs = value)}
            />

            <LabeledSwitch
              hint="Allows the dreamer minion to run a once-per-day memory synthesis pass for this employee."
              label="Allow dreamer minion"
              value={employee.dreams_enabled}
              onChange={(value) => (employee.dreams_enabled = value)}
            />
          </div>
        </div>

        <div className="grid gap-4 md:grid-cols-2">
          <LabeledSelect
            label="Provider"
            options={viewmodel.runtimeProviderOptions}
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
        </div>
      </CardContent>
    </Card>
  )
})

const getPreventEmptyRunsHint = (role: EmployeeRole) => {
  if (role === 'manager' || role === 'ceo') {
    return 'Timed runs start only when this employee has assigned todo/in-progress issues or a direct report has a blocked issue.'
  }

  return 'Timed runs start only when this employee has assigned todo or in-progress issues.'
}
