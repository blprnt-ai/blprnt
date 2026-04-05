import { observer } from 'mobx-react-lite'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { useEmployeeViewmodel } from '../employee.viewmodel'
import { EmployeeLifeFilePanel } from './employee-life-file-panel'
import { EmployeeLifeTree } from './employee-life-tree'

export const EmployeeLifeTab = observer(() => {
  const viewmodel = useEmployeeViewmodel()

  if (viewmodel.lifeErrorMessage && !viewmodel.lifeTree) {
    return (
      <Card className="border-border/60">
        <CardContent className="py-6 text-sm text-destructive">{viewmodel.lifeErrorMessage}</CardContent>
      </Card>
    )
  }

  return (
    <div className="grid gap-4 lg:grid-cols-[280px_minmax(0,1fr)]">
      <Card className="border-border/60">
        <CardHeader>
          <CardTitle>Files</CardTitle>
        </CardHeader>
        <CardContent>
          <EmployeeLifeTree
            nodes={viewmodel.lifeTreeNodes}
            selectedPath={viewmodel.selectedLifePath}
            onSelect={(path) => void viewmodel.selectLifePath(path)}
          />
        </CardContent>
      </Card>
      <div className="space-y-4">
        {viewmodel.lifeErrorMessage && viewmodel.lifeFile ? (
          <Card className="border-destructive/40">
            <CardContent className="py-4 text-sm text-destructive">{viewmodel.lifeErrorMessage}</CardContent>
          </Card>
        ) : null}
        <EmployeeLifeFilePanel />
      </div>
    </div>
  )
})
