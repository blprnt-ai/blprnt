import { observer } from 'mobx-react-lite'
import { Page } from '@/components/layouts/page'
import { Card, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { useEmployeeViewmodel } from '../employee.viewmodel'

export const EmployeeNotFound = observer(() => {
  const viewmodel = useEmployeeViewmodel()

  return (
    <Page className="min-h-screen">
      <Card className="max-w-3xl">
        <CardHeader>
          <CardTitle>Employee unavailable</CardTitle>
          <CardDescription>{viewmodel.errorMessage ?? 'We could not load this employee.'}</CardDescription>
        </CardHeader>
      </Card>
    </Page>
  )
})
