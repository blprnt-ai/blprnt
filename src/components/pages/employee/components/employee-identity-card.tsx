import { Identity } from '@/components/molecules/indentity'
import { LabeledInput } from '@/components/molecules/labeled-input'
import { LabeledSelect } from '@/components/molecules/labeled-select'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { ColoredSpan, type ColorVariant, colors } from '@/components/ui/colors'
import { EmployeeLabel, employeeIcons } from '@/components/ui/employee-label'
import { useEmployeeViewmodel } from '../employee.viewmodel'
import { formatLabel, formatRole } from '../utils'

export const EmployeeIdentityCard = () => {
  const viewmodel = useEmployeeViewmodel()
  const { employee } = viewmodel

  if (!employee) return null

  const showsStatus = viewmodel.showsAgentConfiguration
  const isOwner = viewmodel.isOwnerEmployee

  return (
    <Card className="h-full border-border/60">
      <CardHeader>
        <CardTitle>Profile</CardTitle>
        <CardDescription>Core identity and how this employee shows up across the workspace.</CardDescription>
      </CardHeader>
      <CardContent className="space-y-5">
        <Identity
          className="text-base"
          color={employee.color}
          icon={employee.icon}
          name={employee.name || 'Unnamed employee'}
          size="lg"
        />

        <div className="grid gap-4 md:grid-cols-2">
          <LabeledInput
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

        <div className="grid gap-3 rounded-2xl border border-border/60 bg-muted/20 p-4 text-sm text-muted-foreground sm:grid-cols-2">
          {!isOwner ? (
            <>
              <div>
                <p className="mb-1 text-[11px] font-medium uppercase tracking-[0.18em] text-foreground/70">Title</p>
                <p>{employee.title || 'No title'}</p>
              </div>
              <div>
                <p className="mb-1 text-[11px] font-medium uppercase tracking-[0.18em] text-foreground/70">Role</p>
                <p>{formatRole(employee.role)}</p>
              </div>
            </>
          ) : null}
          <div>
            <p className="mb-1 text-[11px] font-medium uppercase tracking-[0.18em] text-foreground/70">Kind</p>
            <p>{formatLabel(employee.kind)}</p>
          </div>
          {showsStatus && !isOwner ? (
            <div>
              <p className="mb-1 text-[11px] font-medium uppercase tracking-[0.18em] text-foreground/70">Status</p>
              <p>{formatLabel(employee.status)}</p>
            </div>
          ) : null}
          <div>
            <p className="mb-1 text-[11px] font-medium uppercase tracking-[0.18em] text-foreground/70">Employee ID</p>
            <p className="break-all">{employee.id}</p>
          </div>
        </div>
      </CardContent>
    </Card>
  )
}
