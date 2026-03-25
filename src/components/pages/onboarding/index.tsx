import { useState } from 'react'
import { ThemeToggle } from '@/components/molecules/theme-toggle'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from '@/components/ui/card'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { ColoredSpan, type ColorVariant, colors } from '../../ui/colors'
import { EmployeeLabel, employeeIcons } from '../../ui/employee-label'
import { OnboardingViewmodel } from './onboarding.viewmodel'

export const OnboardingPage = () => {
  const [viewmodel] = useState(() => new OnboardingViewmodel())

  const handleNameChange = (value: string) => {
    viewmodel.setName(value)
  }

  const handleIconChange = (value: string | null) => {
    viewmodel.setIcon(value ?? employeeIcons[0].name)
  }

  const handleColorChange = (value: ColorVariant) => {
    viewmodel.setColor(value)
  }

  return (
    <div className="flex h-screen w-screen items-center justify-center relative">
      <div className="absolute top-2 right-2">
        <ThemeToggle />
      </div>

      <Card className="w-full max-w-sm">
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
                  value={viewmodel.name}
                  onChange={(e) => handleNameChange(e.target.value)}
                />
              </div>
              <div className="grid gap-2 w-full grid-cols-2">
                <div className="flex flex-col gap-2">
                  <Label htmlFor="icon">Color</Label>

                  <Select value={viewmodel.color} onValueChange={handleColorChange}>
                    <SelectTrigger className="w-full">
                      <SelectValue placeholder="Select a color">
                        <ColoredSpan className="rounded-full size-4" color={viewmodel.color} />
                        {viewmodel.selectedColor.name}
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

                  <Select value={viewmodel.icon} onValueChange={handleIconChange}>
                    <SelectTrigger className="w-full">
                      <SelectValue placeholder="Select an icon">
                        <EmployeeLabel
                          color={viewmodel.color}
                          Icon={viewmodel.selectedIcon.icon}
                          name={viewmodel.selectedIcon.name}
                        />
                      </SelectValue>
                    </SelectTrigger>
                    <SelectContent>
                      {employeeIcons.map((icon) => (
                        <SelectItem key={icon.name} value={icon.value}>
                          <EmployeeLabel color={viewmodel.color} Icon={icon.icon} name={icon.name} />
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                </div>
              </div>
            </div>
          </form>
        </CardContent>
        <CardFooter className="flex-col gap-2">
          <Button className="w-full" disabled={!viewmodel.isFormValid} type="submit" onClick={() => viewmodel.submit()}>
            Create Owner
          </Button>
        </CardFooter>
      </Card>
    </div>
  )
}
