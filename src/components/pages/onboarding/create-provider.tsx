import { ArrowLeftIcon, ArrowRightIcon, CloudIcon } from 'lucide-react'
import { observer } from 'mobx-react-lite'
import type { ProviderDto } from '@/bindings/ProviderDto'
import { ProviderForm } from '@/components/forms/provider/provider-form'
import { Button } from '@/components/ui/button'
import { Card, CardContent } from '@/components/ui/card'
import { OnboardingStep, useOnboardingViewmodel } from './onboarding.viewmodel'
import { OnboardingCardHeader } from './onboarding-card-header'

export const CreateProvider = observer(() => {
  const viewmodel = useOnboardingViewmodel()

  const handleProviderSaved = (provider: ProviderDto) => {
    void viewmodel.setProvider(provider)
  }

  const leftButtons = !viewmodel.provider?.id ? null : (
    <Button variant="ghost" onClick={() => viewmodel.setStep(OnboardingStep.Owner)}>
      <ArrowLeftIcon className="size-4" /> Back
    </Button>
  )

  const rightButtonText = (
    <>
      <ArrowRightIcon className="size-4" /> Next
    </>
  )

  return (
    <Card className="w-full">
      <OnboardingCardHeader
        icon={<CloudIcon className="size-8" />}
        subtitle="Choose the model provider your agents will use."
        title="Select a provider"
      />
      <CardContent>
        <ProviderForm
          leftButtons={leftButtons}
          provider={viewmodel.provider}
          rightButtonText={rightButtonText}
          onProviderSaved={handleProviderSaved}
        />
      </CardContent>
    </Card>
  )
})
