import { createFileRoute } from '@tanstack/react-router'
import { OnboardingPage } from '@/components/pages/onboarding'

export const Route = createFileRoute('/onboarding/')({
  component: OnboardingPage,
})
