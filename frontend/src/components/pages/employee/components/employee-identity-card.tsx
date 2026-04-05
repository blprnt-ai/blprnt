import { observer } from 'mobx-react-lite'
import type { Employee } from '@/bindings/Employee'
import { LabeledInput } from '@/components/molecules/labeled-input'
import { LabeledSelect } from '@/components/molecules/labeled-select'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { ColoredSpan, type ColorVariant, colors } from '@/components/ui/colors'
import { EmployeeLabel, employeeIcons } from '@/components/ui/employee-label'
import { cn } from '@/lib/utils'
import { AppModel } from '@/models/app.model'
import { useEmployeeViewmodel } from '../employee.viewmodel'
import { canReportTo } from '../utils'

export const EmployeeIdentityCard = observer(() => {
  const viewmodel = useEmployeeViewmodel()
  const { employee } = viewmodel

  if (!employee) return null

  const isOwner = viewmodel.isOwnerEmployee
  const disallowedManagerIds = new Set([employee.id, ...findDescendantIds(employee.id, AppModel.instance.employees)])
  const managerOptions = AppModel.instance.employees
    .filter((candidate) => !disallowedManagerIds.has(candidate.id) && canReportTo(employee.role, candidate.role))
    .map((candidate) => ({ label: candidate.name, value: candidate.id }))
  const selectedManagerName =
    AppModel.instance.resolveEmployeeName(employee.reports_to) ?? (isOwner ? 'No manager assigned' : 'Select a manager')

  return (
    <Card className="h-full border-border/60 z-20">
      <CardHeader>
        <CardTitle>Profile</CardTitle>
      </CardHeader>
      <CardContent className="space-y-5">
        <div className="grid gap-4 md:grid-cols-2">
          <LabeledInput
            className={cn(isOwner && 'col-span-full')}
            label="Name"
            placeholder="CEO"
            value={employee.name}
            onChange={(value) => (employee.name = value)}
          />
          {isOwner ? null : (
            <LabeledInput
              label="Title"
              placeholder="Chief Executive Officer"
              value={employee.title}
              onChange={(value) => (employee.title = value)}
            />
          )}
          {isOwner ? null : (
            <LabeledSelect
              label="Reports to"
              options={managerOptions}
              placeholder="Select a manager"
              selectedValue={selectedManagerName}
              value={employee.reports_to ?? ''}
              onChange={(value) => {
                if (!value) return
                employee.reports_to = value
              }}
            />
          )}
          <LabeledSelect
            label="Color"
            value={employee.color}
            options={colors.map((color) => ({
              label: (
                <span className="flex items-center gap-2">
                  <ColoredSpan className="size-4 rounded-full" color={color.color} />
                  <span>{color.name}</span>
                </span>
              ),
              value: color.color,
            }))}
            selectedValue={
              <>
                <ColoredSpan className="size-4 rounded-full" color={employee.color} />
                {employee.selectedColor.name}
              </>
            }
            onChange={(value) => {
              if (value) employee.color = value as ColorVariant
            }}
          />
          <LabeledSelect
            label="Icon"
            value={employee.icon}
            options={employeeIcons.map((icon) => ({
              label: <EmployeeLabel color={employee.color} Icon={icon.icon} name={icon.name} />,
              value: icon.value,
            }))}
            selectedValue={
              <EmployeeLabel
                color={employee.color}
                Icon={employee.selectedIcon.icon}
                name={employee.selectedIcon.name}
              />
            }
            onChange={(value) => {
              if (value) employee.icon = value
            }}
          />
        </div>
      </CardContent>
    </Card>
  )
})

const findDescendantIds = (employeeId: string | null, employees: Employee[]) => {
  if (!employeeId) return []

  const descendants: string[] = []
  const queue = [employeeId]

  while (queue.length > 0) {
    const managerId = queue.shift()!
    const reports = employees.filter((candidate) => candidate.reports_to === managerId)

    for (const report of reports) {
      descendants.push(report.id)
      queue.push(report.id)
    }
  }

  return descendants
}
