import { LabeledInput } from '@/components/molecules/labeled-input'
import { LabeledSwitch } from '@/components/molecules/labeled-switch'
import { LabeledTextarea } from '@/components/molecules/labeled-textarea'
import { MetadataRow } from '@/components/pages/issue/components/metadata-row'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { useEmployeeViewmodel } from '../employee.viewmodel'

export const EmployeeRuntimeCard = () => {
  const viewmodel = useEmployeeViewmodel()
  const { employee } = viewmodel

  if (!employee) return null

  return (
    <Card>
      <CardHeader>
        <CardTitle>Runtime</CardTitle>
      </CardHeader>
      <CardContent className="space-y-4">
        {viewmodel.isEditing ? (
          <>
            <LabeledInput
              inline
              label="Heartbeat interval"
              value={employee.heartbeat_interval_sec.toString()}
              onChange={(value) => {
                const parsed = Number.parseInt(value, 10)
                if (!Number.isNaN(parsed)) employee.heartbeat_interval_sec = parsed
              }}
            />
            <LabeledInput
              inline
              label="Max concurrent runs"
              value={employee.max_concurrent_runs.toString()}
              onChange={(value) => {
                const parsed = Number.parseInt(value, 10)
                if (!Number.isNaN(parsed)) employee.max_concurrent_runs = parsed
              }}
            />
            <LabeledSwitch
              inline
              label="Wake on demand"
              value={employee.wake_on_demand}
              onChange={(value) => (employee.wake_on_demand = value)}
            />
            <LabeledTextarea
              label="Heartbeat prompt"
              placeholder="Review open work, priorities, and blockers."
              value={employee.heartbeat_prompt}
              onChange={(value) => (employee.heartbeat_prompt = value)}
            />
          </>
        ) : (
          <>
            <MetadataRow label="Heartbeat interval" value={`${employee.heartbeat_interval_sec} seconds`} />
            <MetadataRow label="Max concurrent runs" value={employee.max_concurrent_runs.toString()} />
            <MetadataRow label="Wake on demand" value={employee.wake_on_demand ? 'Enabled' : 'Disabled'} />
            <MetadataRow label="Heartbeat prompt" value={employee.heartbeat_prompt || 'No custom prompt'} />
          </>
        )}
      </CardContent>
    </Card>
  )
}
