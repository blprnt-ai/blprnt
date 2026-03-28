import { LabeledSelect } from '@/components/molecules/labeled-select'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { ColoredSpan, type ColorVariant, colors } from '@/components/ui/colors'
import { EmployeeLabel, employeeIcons } from '@/components/ui/employee-label'
import { useEmployeeViewmodel } from '../employee.viewmodel'

export const EmployeeAppearanceCard = () => {
  const viewmodel = useEmployeeViewmodel()
  const { employee } = viewmodel

  if (!employee) return null

  return (
    <Card>
      <CardHeader>
        <CardTitle>Appearance</CardTitle>
      </CardHeader>
      <CardContent className="grid gap-4 md:grid-cols-2">
        <LabeledSelect
          label="Color"
          value={employee.color}
          options={colors.map((color) => ({
            label: <ColoredSpan className="size-4 rounded-full" color={color.color} />,
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
            <EmployeeLabel color={employee.color} Icon={employee.selectedIcon.icon} name={employee.selectedIcon.name} />
          }
          onChange={(value) => {
            if (value) employee.icon = value
          }}
        />
      </CardContent>
    </Card>
  )
}
