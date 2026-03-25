import { Card, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'

export const CreateIssue = () => {
  return (
    <Card className="w-full max-w-lg">
      <CardHeader>
        <CardTitle>Create a new issue</CardTitle>
        <CardDescription>Enter the title and description of your issue</CardDescription>
      </CardHeader>
    </Card>
  )
}
