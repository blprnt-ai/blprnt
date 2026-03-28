import { LabeledInput } from '@/components/molecules/labeled-input'
import { LabeledSwitch } from '@/components/molecules/labeled-switch'
import { MarkdownEditor } from '@/components/organisms/markdown-editor'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Label } from '@/components/ui/label'
import { useEmployeeViewmodel } from '../employee.viewmodel'

export const EmployeeRuntimeCard = () => {
  const viewmodel = useEmployeeViewmodel()
  const { employee } = viewmodel

  if (!employee) return null

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
          label="Wake on demand"
          hint="Turn this on if the coordinator should wake the employee only when work arrives."
          value={employee.wake_on_demand}
          onChange={(value) => (employee.wake_on_demand = value)}
        />
      </CardContent>
    </Card>
  )
}
