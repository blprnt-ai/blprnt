import type { EmployeeStatus } from '@/bindings/EmployeeStatus'
import { Identity } from '@/components/molecules/indentity'
import { LabeledInput } from '@/components/molecules/labeled-input'
import { LabeledSelect } from '@/components/molecules/labeled-select'
import { MetadataRow } from '@/components/pages/issue/components/metadata-row'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { useEmployeeViewmodel } from '../employee.viewmodel'
import { formatLabel, formatRole } from '../utils'

const statusOptions: { label: string; value: EmployeeStatus }[] = [
  { label: 'Idle', value: 'idle' },
  { label: 'Running', value: 'running' },
  { label: 'Terminated', value: 'terminated' },
]

export const EmployeeIdentityCard = () => {
  const viewmodel = useEmployeeViewmodel()
  const { employee } = viewmodel

  if (!employee) return null

  return (
    <Card>
      <CardHeader>
        <CardTitle>Identity</CardTitle>
      </CardHeader>
      <CardContent className="space-y-4">
        <Identity
          className="text-base"
          color={employee.color}
          icon={employee.icon}
          name={employee.name || 'Unnamed employee'}
          size="lg"
        />

        {viewmodel.isEditing ? (
          <div className="grid gap-4 md:grid-cols-2">
            <LabeledInput label="Name" value={employee.name} onChange={(value) => (employee.name = value)} />
            <LabeledInput label="Title" value={employee.title} onChange={(value) => (employee.title = value)} />
            <LabeledInput label="Role" value={viewmodel.roleValue} onChange={viewmodel.setRole} />
            <LabeledSelect
              label="Status"
              options={statusOptions}
              selectedValue={formatLabel(employee.status)}
              value={employee.status}
              onChange={(value) => {
                if (value) employee.status = value as EmployeeStatus
              }}
            />
          </div>
        ) : (
          <div className="grid gap-4 md:grid-cols-2">
            <MetadataRow label="Title" value={employee.title || 'No title'} />
            <MetadataRow label="Role" value={formatRole(employee.role)} />
            <MetadataRow label="Kind" value={formatLabel(employee.kind)} />
            <MetadataRow label="Status" value={formatLabel(employee.status)} />
          </div>
        )}
      </CardContent>
    </Card>
  )
}
