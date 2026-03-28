import type { Provider } from '@/bindings/Provider'
import { LabeledSelect } from '@/components/molecules/labeled-select'
import { SlugSelect } from '@/components/organisms/slug-select'
import { MetadataRow } from '@/components/pages/issue/components/metadata-row'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
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
    <Card>
      <CardHeader>
        <CardTitle>Connection</CardTitle>
      </CardHeader>
      <CardContent className="space-y-4">
        {viewmodel.isEditing ? (
          <>
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
          </>
        ) : (
          <>
            <MetadataRow label="Provider" value={formatProvider(employee.provider)} />
            <MetadataRow label="Model" value={employee.slug || 'No model selected'} />
          </>
        )}
      </CardContent>
    </Card>
  )
}
