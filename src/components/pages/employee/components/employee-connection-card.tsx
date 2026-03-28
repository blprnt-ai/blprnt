import type { Provider } from '@/bindings/Provider'
import { LabeledSelect } from '@/components/molecules/labeled-select'
import { SlugSelect } from '@/components/organisms/slug-select'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { useEmployeeViewmodel } from '../employee.viewmodel'
import { formatProvider } from '../utils'

const providerOptions: { label: string; value: Provider }[] = [
  { label: 'Anthropic', value: 'anthropic' },
  { label: 'Claude Code', value: 'claude_code' },
  { label: 'Codex', value: 'codex' },
  { label: 'Mock', value: 'mock' },
  { label: 'OpenAI', value: 'openai' },
  { label: 'OpenRouter', value: 'open_router' },
]

export const EmployeeConnectionCard = () => {
  const viewmodel = useEmployeeViewmodel()
  const { employee } = viewmodel

  if (!employee) return null

  return (
    <Card className="border-border/60">
      <CardHeader>
        <CardTitle>Provider</CardTitle>
        <CardDescription>
          Point this employee at the runtime provider and model it should use by default.
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        <LabeledSelect
          label="Provider"
          options={providerOptions}
          selectedValue={formatProvider(employee.provider)}
          value={employee.provider}
          onChange={(value) => {
            if (value) viewmodel.setProvider(value as Provider)
          }}
        />
        <SlugSelect
          provider={employee.provider}
          slug={employee.slug}
          onChange={(value) => (employee.slug = value ?? '')}
        />
      </CardContent>
    </Card>
  )
}
