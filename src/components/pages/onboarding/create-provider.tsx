import { ProviderForm } from '@/components/forms/provider/provider-form'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { OnboardingStep, useOnboardingViewmodel } from './onboarding.viewmodel'

export const CreateProvider = () => {
  const viewmodel = useOnboardingViewmodel()

  const handleProviderSaved = () => viewmodel.setStep(OnboardingStep.Project)

  return (
    <Card className="w-full max-w-xl">
      <CardHeader>
        <CardTitle>Create Provider</CardTitle>
      </CardHeader>
      <CardContent>
        <ProviderForm onProviderSaved={handleProviderSaved} />
      </CardContent>
    </Card>
  )
}
