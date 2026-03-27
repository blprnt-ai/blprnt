import { Page } from '@/components/layouts/page'
import { Card, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { useIssueViewmodel } from '../issue.viewmodel'

export const IssueNotFound = () => {
  const viewmodel = useIssueViewmodel()

  return (
    <Page className="min-h-screen">
      <Card className="max-w-3xl">
        <CardHeader>
          <CardTitle>Issue unavailable</CardTitle>
          <CardDescription>{viewmodel.errorMessage ?? 'We could not load this issue.'}</CardDescription>
        </CardHeader>
      </Card>
    </Page>
  )
}
