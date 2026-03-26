import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from '@/components/ui/card'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Textarea } from '@/components/ui/textarea'
import { useOnboardingViewmodel } from './onboarding.viewmodel'

export const CreateIssue = () => {
  const viewmodel = useOnboardingViewmodel()

  return (
    <Card className="w-full max-w-lg">
      <CardHeader>
        <CardTitle>Create a new issue</CardTitle>
        <CardDescription>Enter the title and description of your issue</CardDescription>
      </CardHeader>
      <CardContent>
        <form>
          <div className="flex flex-col gap-6">
            <div className="flex flex-col gap-2">
              <Label htmlFor="issue-title">Title</Label>
              <Input
                required
                id="issue-title"
                placeholder="Kick off roadmap planning"
                type="text"
                value={viewmodel.issue.title}
                onChange={(e) => {
                  viewmodel.issue.title = e.target.value
                }}
              />
            </div>
            <div className="flex flex-col gap-2">
              <Label htmlFor="issue-description">Description</Label>
              <Textarea
                required
                id="issue-description"
                placeholder="Outline the first priorities for the team and capture any immediate blockers."
                value={viewmodel.issue.description}
                onChange={(e) => {
                  viewmodel.issue.description = e.target.value
                }}
              />
            </div>
            {viewmodel.issue.assignee && (
              <p className="text-sm text-muted-foreground">This issue will be assigned to the CEO you just created.</p>
            )}
          </div>
        </form>
      </CardContent>
      <CardFooter className="flex justify-end">
        <Button disabled={!viewmodel.issue.isValid} type="submit" onClick={() => viewmodel.saveIssue()}>
          Create Issue
        </Button>
      </CardFooter>
    </Card>
  )
}
