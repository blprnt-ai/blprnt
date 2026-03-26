import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from '@/components/ui/card'
import { ColoredSpan, type ColorVariant, colors } from '@/components/ui/colors'
import { EmployeeLabel, employeeIcons } from '@/components/ui/employee-label'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { useOnboardingViewmodel } from './onboarding.viewmodel'

const fallbackIcon = employeeIcons.find((icon) => icon.default)?.value ?? employeeIcons[0].value

export const CreateCeo = () => {
  const viewmodel = useOnboardingViewmodel()

  const handleNameChange = (value: string) => {
    viewmodel.ceo.name = value
  }

  const handleIconChange = (value: string | null) => {
    viewmodel.ceo.icon = value ?? fallbackIcon
  }

  const handleColorChange = (value: ColorVariant | null) => {
    viewmodel.ceo.color = value ?? colors[0].color
  }

  return (
    <Card className="w-full max-w-lg">
      <CardHeader>
        <CardTitle>Create a new CEO</CardTitle>
        <CardDescription>Enter the name and select a color and icon to create a new CEO</CardDescription>
      </CardHeader>
      <CardContent>
        <form>
          <div className="flex flex-col gap-6">
            <div className="flex flex-col gap-2">
              <Label>Name</Label>
              <Input
                required
                placeholder="Theon Rust"
                type="text"
                value={viewmodel.ceo.name}
                onChange={(e) => handleNameChange(e.target.value)}
              />
            </div>
            <div className="grid gap-2 w-full grid-cols-2">
              <div className="flex flex-col gap-2">
                <Label>Color</Label>

                <Select value={viewmodel.ceo.color} onValueChange={handleColorChange}>
                  <SelectTrigger className="w-full">
                    <SelectValue placeholder="Select a color">
                      <ColoredSpan className="rounded-full size-4" color={viewmodel.ceo.color} />
                      {viewmodel.ceo.selectedColor.name}
                    </SelectValue>
                  </SelectTrigger>
                  <SelectContent>
                    {colors.map((color) => (
                      <SelectItem key={color.color} value={color.color}>
                        <ColoredSpan className="rounded-full size-4" color={color.color} />
                        <span>{color.name}</span>
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>
              <div className="flex flex-col gap-2">
                <Label>Icon</Label>

                <Select value={viewmodel.ceo.icon} onValueChange={handleIconChange}>
                  <SelectTrigger className="w-full">
                    <SelectValue placeholder="Select an icon">
                      <EmployeeLabel
                        color={viewmodel.ceo.color}
                        Icon={viewmodel.ceo.selectedIcon.icon}
                        name={viewmodel.ceo.selectedIcon.name}
                      />
                    </SelectValue>
                  </SelectTrigger>
                  <SelectContent>
                    {employeeIcons.map((icon) => (
                      <SelectItem key={icon.name} value={icon.value}>
                        <EmployeeLabel color={viewmodel.ceo.color} Icon={icon.icon} name={icon.name} />
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>
            </div>
          </div>
        </form>
      </CardContent>
      <CardFooter className="flex justify-end">
        <Button disabled={viewmodel.ceo.isDirty} type="submit" onClick={viewmodel.saveCeo}>
          Create CEO
        </Button>
      </CardFooter>
    </Card>
  )
}
