import { EmployeeOnboardingCard } from './employee-onboarding-card'
import { useOnboardingViewmodel } from './onboarding.viewmodel'

export const OwnerSignup = () => {
  const viewmodel = useOnboardingViewmodel()

  return (
    <EmployeeOnboardingCard
      description="Enter your name and select a color and icon to get started"
      employee={viewmodel.owner}
      isSubmitDisabled={!viewmodel.owner.isDirty}
      namePlaceholder="Beff Jezos"
      submitLabel="Create Owner"
      title={
        <>
          Welcome to
          <span className="text-primary"> blprnt</span>
        </>
      }
      onSubmit={() => viewmodel.saveOwner()}
    />
  )
}
