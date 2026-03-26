import { Navigate } from '@tanstack/react-router'
import { BrainIcon, CloudIcon, FolderIcon, RocketIcon, UserIcon } from 'lucide-react'
import { useEffect, useState } from 'react'
import { Page } from '@/components/layouts/page'
import { ThemeToggle } from '@/components/molecules/theme-toggle'
import { AppLoader } from '@/components/organisms/app-loader'
import { Tabs, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { CreateCeo } from './create-ceo'
import { CreateIssue } from './create-issue'
import { CreateProject } from './create-project'
import { CreateProvider } from './create-provider'
import {
  OnboardingStep,
  OnboardingViewmodel,
  OnboardingViewmodelContext,
  useOnboardingViewmodel,
} from './onboarding.viewmodel'
import { OwnerSignup } from './owner-signup'

export const OnboardingPage = () => {
  const [viewmodel, setViewmodel] = useState<OnboardingViewmodel | null>(null)

  useEffect(() => {
    const viewmodel = new OnboardingViewmodel()
    viewmodel.init().then(() => setViewmodel(viewmodel))
  }, [])

  if (!viewmodel) return <AppLoader />

  return (
    <Page>
      <OnboardingViewmodelContext.Provider value={viewmodel}>
        <div className="flex h-screen w-screen items-center justify-center relative">
          <div className="absolute top-2 right-2 opacity-30 hover:opacity-100 transition-opacity duration-300">
            <ThemeToggle />
          </div>

          <div className="flex flex-col gap-2 w-full max-w-xl">
            <OnboardingTabs />

            <OnboardingSteps />
          </div>
        </div>
      </OnboardingViewmodelContext.Provider>
    </Page>
  )
}

export const OnboardingTabs = () => {
  const viewmodel = useOnboardingViewmodel()

  return (
    <Tabs className="border-b" value={viewmodel.step} onValueChange={(value) => viewmodel.setStep(value)}>
      <TabsList variant="line">
        <TabsTrigger value={OnboardingStep.Owner}>
          <UserIcon className="size-4" />
          Owner
        </TabsTrigger>
        <TabsTrigger value={OnboardingStep.Provider}>
          <CloudIcon className="size-4" />
          Provider
        </TabsTrigger>
        <TabsTrigger value={OnboardingStep.Project}>
          <FolderIcon className="size-4" />
          Project
        </TabsTrigger>
        <TabsTrigger value={OnboardingStep.Ceo}>
          <BrainIcon className="size-4" />
          Ceo
        </TabsTrigger>
        <TabsTrigger value={OnboardingStep.Issue}>
          <RocketIcon className="size-4" />
          Launch
        </TabsTrigger>
      </TabsList>
    </Tabs>
  )
}

export const OnboardingSteps = () => {
  const viewmodel = useOnboardingViewmodel()

  return (
    <>
      {viewmodel.step === OnboardingStep.Owner && <OwnerSignup />}
      {viewmodel.step === OnboardingStep.Provider && <CreateProvider />}
      {viewmodel.step === OnboardingStep.Project && <CreateProject />}
      {viewmodel.step === OnboardingStep.Ceo && <CreateCeo />}
      {viewmodel.step === OnboardingStep.Issue && <CreateIssue />}
      {viewmodel.step === OnboardingStep.Done && <Navigate replace to="/" />}
    </>
  )
}
