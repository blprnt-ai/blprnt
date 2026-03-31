import type { Provider } from '@/bindings/Provider'
import { LabeledInput } from '@/components/molecules/labeled-input'
import { LabeledSelect } from '@/components/molecules/labeled-select'
import { SlugSelect } from '@/components/organisms/slug-select'
import { formatProvider, formatRole } from '@/components/pages/employee/utils'
import { ColoredSpan, type ColorVariant, colors } from '@/components/ui/colors'
import { EmployeeLabel, employeeIcons } from '@/components/ui/employee-label'
import type { EmployeeFormViewmodel } from './employee-form.viewmodel'

const roleOptions: Array<{ label: string; value: 'ceo' | 'manager' | 'staff' }> = [
  { label: 'CEO', value: 'ceo' },
  { label: 'Manager', value: 'manager' },
  { label: 'Staff', value: 'staff' },
]

const providerOptions: { label: string; value: Provider }[] = [
  { label: 'Anthropic', value: 'anthropic' },
  { label: 'Claude Code', value: 'claude_code' },
  { label: 'Codex', value: 'codex' },
  { label: 'OpenAI', value: 'openai' },
  { label: 'OpenRouter', value: 'open_router' },
]

interface EmployeeFormFieldsProps {
  viewmodel: EmployeeFormViewmodel
}

export const EmployeeFormFields = ({ viewmodel }: EmployeeFormFieldsProps) => {
  const { employee } = viewmodel

  return (
    <div className="flex min-h-0 flex-1 flex-col gap-5 overflow-y-auto px-4 py-2">
      <div className="grid gap-4 md:grid-cols-2">
        <LabeledInput
          label="Name"
          placeholder="Head of Product"
          value={employee.name}
          onChange={(value) => (employee.name = value)}
        />
        <LabeledInput
          label="Title"
          placeholder="Chief of Staff"
          value={employee.title}
          onChange={(value) => (employee.title = value)}
        />
      </div>

      <div className="grid gap-4 md:grid-cols-2">
        <LabeledSelect
          label="Role"
          options={roleOptions}
          selectedValue={formatRole(employee.role)}
          value={typeof employee.role === 'string' ? employee.role : employee.role.custom}
          onChange={(value) => {
            if (value) employee.role = value as 'ceo' | 'manager' | 'staff'
          }}
        />
        <LabeledSelect
          label="Provider"
          options={providerOptions}
          selectedValue={formatProvider(employee.provider)}
          value={employee.provider}
          onChange={(value) => {
            if (!value) return
            viewmodel.setProvider(value)
          }}
        />
      </div>

      <div className="grid gap-4 md:grid-cols-2">
        <LabeledSelect
          label="Color"
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
          value={employee.color}
          onChange={(value) => {
            if (value) employee.color = value as ColorVariant
          }}
        />
        <LabeledSelect
          label="Icon"
          options={employeeIcons.map((icon) => ({
            label: <EmployeeLabel color={employee.color} Icon={icon.icon} name={icon.name} />,
            value: icon.value,
          }))}
          selectedValue={
            <EmployeeLabel color={employee.color} Icon={employee.selectedIcon.icon} name={employee.selectedIcon.name} />
          }
          value={employee.icon}
          onChange={(value) => {
            if (value) employee.icon = value
          }}
        />
      </div>

      <SlugSelect
        provider={employee.provider}
        slug={employee.slug}
        onChange={(value) => viewmodel.setSlug(value ?? '')}
      />
    </div>
  )
}
