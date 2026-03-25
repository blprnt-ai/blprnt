import { makeAutoObservable } from 'mobx'
import { toast } from 'sonner'
import { type ColorVariant, colors } from '@/components/ui/colors'
import { employeeIcons } from '@/components/ui/employee-label'
import { employeesApi } from '@/lib/api/employees'
import { AppModel } from '@/models/app.model'

export class OnboardingViewmodel {
  private appModel = AppModel.instance
  public name = ''
  public icon = employeeIcons.find((i) => i.default)?.name ?? employeeIcons[0].name
  public color = colors[0].color

  constructor() {
    makeAutoObservable(this)
  }

  public setName(name: string) {
    this.name = name
  }

  public setIcon(icon: string) {
    this.icon = icon
  }

  public setColor(color: ColorVariant) {
    this.color = color
  }

  public get selectedColor() {
    return colors.find((c) => c.color === this.color)!
  }

  public get selectedIcon() {
    return employeeIcons.find((i) => i.name === this.icon)!
  }

  public get isFormValid() {
    return this.name.length > 0
  }

  public async submit() {
    if (!this.name || !this.icon || !this.color) return

    try {
      const owner = await employeesApi.ownerOnboarding({
        color: this.color,
        icon: this.icon,
        name: this.name,
      })

      this.appModel.setOwner(owner)
    } catch (error) {
      console.error(error)
      toast.error('Failed to onboard owner. Please try again.')
    }
  }
}
