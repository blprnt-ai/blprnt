import { ArrowLeftIcon, ArrowRightIcon, BrainIcon } from 'lucide-react'
import { LabeledInput } from '@/components/molecules/labeled-input'
import { LabeledSwitch } from '@/components/molecules/labeled-switch'
import { LabeledTextarea } from '@/components/molecules/labeled-textarea'
import { SlugSelect } from '@/components/organisms/slug-select'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardFooter } from '@/components/ui/card'
import { OnboardingStep, useOnboardingViewmodel } from './onboarding.viewmodel'
import { OnboardingCardHeader } from './onboarding-card-header'

export const CreateCeo = () => {
  const viewmodel = useOnboardingViewmodel()

  const handleNameChange = (value: string) => {
    viewmodel.ceo.name = value
  }

  const handleHeartbeatPromptChange = (value: string) => {
    viewmodel.ceo.heartbeat_prompt = value
  }

  const handleWakeOnDemandChange = (value: boolean) => {
    viewmodel.ceo.wake_on_demand = value
  }

  const handleMaxConcurrentRunsChange = (value: string) => {
    viewmodel.ceo.max_concurrent_runs = parseInt(value, 10)
  }

  const handleHeartbeatIntervalChange = (value: string) => {
    viewmodel.ceo.heartbeat_interval_sec = parseInt(value, 10)
  }

  const handleProviderConfigChange = (slug: string | null) => {
    viewmodel.ceo.slug = slug ?? ''
  }

  const handleSave = async (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault()

    await viewmodel.saveCeo()
  }

  return (
    <Card className="w-full">
      <form onSubmit={handleSave}>
        <OnboardingCardHeader
          icon={<BrainIcon className="size-8" />}
          subtitle="Configure your CEO's settings and connect them to the provider."
          title="Hire your CEO"
        />

        <CardContent>
          <div className="flex flex-col gap-6">
            <LabeledInput label="Name" value={viewmodel.ceo.name} onChange={handleNameChange} />

            <LabeledTextarea
              hint="This prompt is in addition to the blprnt heartbeat and the employee HEARTBEAT.md file."
              placeholder="We're building a dirt farm."
              value={viewmodel.ceo.heartbeat_prompt}
              label={
                <>
                  Heartbeat Prompt<span className="text-xs text-muted-foreground"> (optional)</span>
                </>
              }
              onChange={handleHeartbeatPromptChange}
            />

            <LabeledSwitch
              inline
              hint="If disbaled, CEO will not wake up on assigned tasks."
              label="Wake on demand"
              value={viewmodel.ceo.wake_on_demand}
              onChange={handleWakeOnDemandChange}
            />

            <LabeledInput
              inline
              hint="The number of concurrent runs the CEO can have. Minimum is 1."
              label="Max concurrent runs"
              size="sm"
              value={viewmodel.ceo.max_concurrent_runs.toString()}
              onChange={handleMaxConcurrentRunsChange}
            />

            <LabeledInput
              inline
              hint="The interval (in seconds) at which the CEO will wake up to check for new tasks."
              label="Heartbeat interval"
              size="sm"
              value={viewmodel.ceo.heartbeat_interval_sec.toString()}
              onChange={handleHeartbeatIntervalChange}
            />

            <SlugSelect
              provider={viewmodel.provider.provider}
              slug={viewmodel.ceo.slug}
              onChange={handleProviderConfigChange}
            />
          </div>
        </CardContent>
        <CardFooter className="flex justify-between">
          <Button variant="ghost" onClick={() => viewmodel.setStep(OnboardingStep.Project)}>
            <ArrowLeftIcon className="size-4" /> Back
          </Button>

          <Button disabled={!viewmodel.ceo.isIdentityValid} type="submit">
            <ArrowRightIcon className="size-4" /> Next
          </Button>
        </CardFooter>
      </form>
    </Card>
  )
}
