import { MetadataRow } from '@/components/pages/issue/components/metadata-row'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { AppModel } from '@/models/app.model'
import { useEmployeeViewmodel } from '../employee.viewmodel'

export const EmployeeHierarchyCard = () => {
  const viewmodel = useEmployeeViewmodel()
  const { employee } = viewmodel

  if (!employee) return null

  const reportsTo = AppModel.instance.resolveEmployeeName(viewmodel.reportsTo) ?? 'No manager'
  const chainOfCommand =
    viewmodel.chainOfCommand.length > 0
      ? viewmodel.chainOfCommand.map((entry) => entry.name).join(' -> ')
      : 'No chain of command'

  return (
    <Card>
      <CardHeader>
        <CardTitle>Hierarchy</CardTitle>
      </CardHeader>
      <CardContent className="space-y-4">
        <MetadataRow label="Reports to" value={reportsTo} />
        <MetadataRow label="Chain of command" value={chainOfCommand} />
      </CardContent>
    </Card>
  )
}
