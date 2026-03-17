import { LabeledTextarea } from '@/components/molecules/labeled-textarea'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { useEmployeeViewmodel } from '../employee.viewmodel'
import { formatCapabilities } from '../utils'

export const EmployeeCapabilitiesCard = () => {
  const viewmodel = useEmployeeViewmodel()
  const { employee } = viewmodel

  if (!employee) return null

  return (
    <Card>
      <CardHeader>
        <CardTitle>Capabilities</CardTitle>
      </CardHeader>
      <CardContent>
        {viewmodel.isEditing ? (
          <LabeledTextarea
            hint="Separate capabilities with commas."
            label="Capabilities"
            placeholder="planning, strategy, hiring"
            value={viewmodel.capabilitiesValue}
            onChange={viewmodel.setCapabilities}
          />
        ) : (
          <p className="text-sm leading-6 text-muted-foreground">{formatCapabilities(employee.capabilities)}</p>
        )}
      </CardContent>
    </Card>
  )
}
