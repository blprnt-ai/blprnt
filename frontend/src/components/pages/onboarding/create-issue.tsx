import { ArrowLeftIcon, RocketIcon } from 'lucide-react'
import { observer } from 'mobx-react-lite'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardFooter } from '@/components/ui/card'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Textarea } from '@/components/ui/textarea'
import { OnboardingStep, useOnboardingViewmodel } from './onboarding.viewmodel'
import { OnboardingCardHeader } from './onboarding-card-header'

export const CreateIssue = observer(() => {
  const viewmodel = useOnboardingViewmodel()

  const handleSave = async (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault()

    await viewmodel.saveIssue()
  }

  return (
    <Card className="w-full">
      <form onSubmit={handleSave}>
        <OnboardingCardHeader
          icon={<RocketIcon className="size-8" />}
          subtitle="Create your first issue to get started."
          title="Launch your project"
        />
        <CardContent>
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
                minRows={12}
                placeholder="Outline the first priorities for the team and capture any immediate blockers."
                value={viewmodel.issue.description}
                onChange={(e) => {
                  viewmodel.issue.description = e.target.value
                }}
              />
            </div>

            <p className="text-sm text-muted-foreground font-light">
              This issue will be assigned to the CEO you just created.
            </p>
          </div>
        </CardContent>
        <CardFooter className="flex justify-between">
          <Button variant="ghost" onClick={() => viewmodel.setStep(OnboardingStep.Ceo)}>
            <ArrowLeftIcon className="size-4" /> Back
          </Button>
          <Button disabled={!viewmodel.issue.isValid} type="submit">
            <RocketIcon className="size-4" /> Launch
          </Button>
        </CardFooter>
      </form>
    </Card>
  )
})
