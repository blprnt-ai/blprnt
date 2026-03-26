import { useEffect, useState } from 'react'
import { ThemeToggle } from '@/components/molecules/theme-toggle'
import { CreateCeo } from './create-ceo'
import { CreateIssue } from './create-issue'
import { CreateProject } from './create-project'
import { CreateProvider } from './create-provider'
import { OnboardingStep, OnboardingViewmodel, OnboardingViewmodelContext } from './onboarding.viewmodel'
import { OwnerSignup } from './owner-signup'

export const OnboardingPage = () => {
  const [viewmodel] = useState(() => new OnboardingViewmodel())

  // biome-ignore lint/correctness/useExhaustiveDependencies: only run once
  useEffect(() => {
    viewmodel.init()
  }, [])

  return (
    <OnboardingViewmodelContext.Provider value={viewmodel}>
      <div className="flex h-screen w-screen items-center justify-center relative">
        <div className="absolute top-2 right-2 opacity-30 hover:opacity-100 transition-opacity duration-300">
          <ThemeToggle />
        </div>

        {viewmodel.step === OnboardingStep.Owner && <OwnerSignup />}
        {viewmodel.step === OnboardingStep.Provider && <CreateProvider />}
        {viewmodel.step === OnboardingStep.Project && <CreateProject />}
        {viewmodel.step === OnboardingStep.Ceo && <CreateCeo />}
        {viewmodel.step === OnboardingStep.Issue && <CreateIssue />}
      </div>
    </OnboardingViewmodelContext.Provider>
  )
}
