import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from '@/components/ui/card'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { ColoredSpan, type ColorVariant, colors } from '../../ui/colors'
import { EmployeeLabel, employeeIcons } from '../../ui/employee-label'
import { useOnboardingViewmodel } from './onboarding.viewmodel'

export const OwnerSignup = () => {
  const viewmodel = useOnboardingViewmodel()

  const handleNameChange = (value: string) => {
    viewmodel.owner.name = value
  }

  const handleIconChange = (value: string | null) => {
    viewmodel.owner.icon = value ?? employeeIcons[0].name
  }

  const handleColorChange = (value: ColorVariant | null) => {
    viewmodel.owner.color = value ?? colors[0].color
  }

  return (
    <Card className="w-full max-w-lg">
      <CardHeader>
        <CardTitle>
          Welcome to
          <span className="text-primary"> blprnt</span>
        </CardTitle>
        <CardDescription>Enter your name and select a color and icon to get started</CardDescription>
      </CardHeader>
      <CardContent>
        <form>
          <div className="flex flex-col gap-6">
            <div className="flex flex-col gap-2">
              <Label htmlFor="email">Name</Label>
              <Input
                required
                id="name"
                placeholder="Beff Jezos"
                type="text"
                value={viewmodel.owner.name}
                onChange={(e) => handleNameChange(e.target.value)}
              />
            </div>
            <div className="grid gap-2 w-full grid-cols-2">
              <div className="flex flex-col gap-2">
                <Label htmlFor="icon">Color</Label>

                <Select value={viewmodel.owner.color} onValueChange={handleColorChange}>
                  <SelectTrigger className="w-full">
                    <SelectValue placeholder="Select a color">
                      <ColoredSpan className="rounded-full size-4" color={viewmodel.owner.color} />
                      {viewmodel.owner.selectedColor.name}
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
                <Label htmlFor="icon">Icon</Label>

                <Select value={viewmodel.owner.icon} onValueChange={handleIconChange}>
                  <SelectTrigger className="w-full">
                    <SelectValue placeholder="Select an icon">
                      <EmployeeLabel
                        color={viewmodel.owner.color}
                        Icon={viewmodel.owner.selectedIcon.icon}
                        name={viewmodel.owner.selectedIcon.name}
                      />
                    </SelectValue>
                  </SelectTrigger>
                  <SelectContent>
                    {employeeIcons.map((icon) => (
                      <SelectItem key={icon.name} value={icon.value}>
                        <EmployeeLabel color={viewmodel.owner.color} Icon={icon.icon} name={icon.name} />
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
        <Button disabled={!viewmodel.owner.isDirty} type="submit" onClick={() => viewmodel.saveOwner()}>
          Create Owner
        </Button>
      </CardFooter>
    </Card>
  )
}
