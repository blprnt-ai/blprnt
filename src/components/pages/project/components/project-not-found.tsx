import { observer } from 'mobx-react-lite'
import { Page } from '@/components/layouts/page'
import { Card, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { useProjectViewmodel } from '../project.viewmodel'

export const ProjectNotFound = observer(() => {
  const viewmodel = useProjectViewmodel()

  return (
    <Page className="min-h-screen">
      <Card className="max-w-3xl">
        <CardHeader>
          <CardTitle>Project unavailable</CardTitle>
          <CardDescription>{viewmodel.errorMessage ?? 'We could not load this project.'}</CardDescription>
        </CardHeader>
      </Card>
    </Page>
  )
})
