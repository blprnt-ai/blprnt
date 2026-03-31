import { ArrowRightIcon, UserIcon } from 'lucide-react'
import { observer } from 'mobx-react-lite'
import { LabeledInput } from '@/components/molecules/labeled-input'
import { LabeledSelect } from '@/components/molecules/labeled-select'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardFooter } from '@/components/ui/card'
import { ColoredSpan, type ColorVariant, colors, fallbackColor } from '@/components/ui/colors'
import { EmployeeLabel, employeeIcons, fallbackIcon } from '@/components/ui/employee-label'
import { useOnboardingViewmodel } from './onboarding.viewmodel'
import { OnboardingCardHeader } from './onboarding-card-header'

export const OwnerSignup = observer(() => {
  const viewmodel = useOnboardingViewmodel()

  const handleNameChange = (value: string) => {
    viewmodel.owner.name = value
  }

  const handleIconChange = (value: string | null) => {
    viewmodel.owner.icon = value ?? fallbackIcon
  }

  const handleColorChange = (value: ColorVariant | null) => {
    viewmodel.owner.color = value ?? fallbackColor
  }

  const handleSave = async (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault()

    await viewmodel.saveOwner()
  }

  return (
    <Card className="w-full">
      <form onSubmit={handleSave}>
        <OnboardingCardHeader
          icon={<UserIcon className="size-8" />}
          subtitle="Create your owner profile."
          title={
            <>
              Welcome to
              <span className="text-primary"> blprnt</span>
            </>
          }
        />
        <CardContent>
          <div className="flex flex-col gap-6">
            <LabeledInput
              label="Name"
              placeholder="Beff Jezos"
              value={viewmodel.owner.name}
              onChange={handleNameChange}
            />
            <div className="grid gap-2 w-full grid-cols-2">
              <LabeledSelect
                label="Color"
                placeholder="Select a color"
                value={viewmodel.owner.color}
                options={colors.map((color) => ({
                  label: (
                    <>
                      <ColoredSpan className="rounded-full size-4" color={color.color} />
                      <span>{color.name}</span>
                    </>
                  ),
                  value: color.color,
                }))}
                selectedValue={
                  <>
                    <ColoredSpan className="rounded-full size-4" color={viewmodel.owner.color} />
                    {viewmodel.owner.selectedColor.name}
                  </>
                }
                onChange={handleColorChange}
              />

              <div className="flex flex-col gap-2">
                <LabeledSelect
                  label="Icon"
                  placeholder="Select an icon"
                  value={viewmodel.owner.icon}
                  options={employeeIcons.map((icon) => ({
                    label: <EmployeeLabel color={viewmodel.owner.color} Icon={icon.icon} name={icon.name} />,
                    value: icon.value,
                  }))}
                  selectedValue={
                    <EmployeeLabel
                      color={viewmodel.owner.color}
                      Icon={viewmodel.owner.selectedIcon.icon}
                      name={viewmodel.owner.selectedIcon.name}
                    />
                  }
                  onChange={handleIconChange}
                />
              </div>
            </div>
          </div>
        </CardContent>
        <CardFooter className="flex gap-2 justify-end">
          <Button disabled={!viewmodel.owner.isOwnerValid} type="submit">
            <ArrowRightIcon className="size-4" /> Next
          </Button>
        </CardFooter>
      </form>
    </Card>
  )
})
