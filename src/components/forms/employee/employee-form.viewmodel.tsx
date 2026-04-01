import { makeAutoObservable, runInAction } from 'mobx'
import type { Employee } from '@/bindings/Employee'
import type { Provider } from '@/bindings/Provider'
import type { Skill } from '@/bindings/Skill'
import { employeesApi } from '@/lib/api/employees'
import { skillsApi } from '@/lib/api/skills'
import { EmployeeModel } from '@/models/employee.model'

export class EmployeeFormViewmodel {
  public availableSkills: Skill[] = []
  public isOpen = false
  public isSaving = false
  public isSkillsLoading = false
  public skillsErrorMessage: string | null = null
  public employee: EmployeeModel = new EmployeeModel()
  private onCreated?: (employee: Employee) => Promise<void> | void

  constructor(onCreated?: (employee: Employee) => Promise<void> | void) {
    this.onCreated = onCreated
    makeAutoObservable(this)
  }

  public get canSave() {
    return this.employee.isIdentityValid && !this.isSaving
  }

  public open = () => {
    this.reset()
    this.isOpen = true
    void this.loadSkills()
  }

  public close = () => {
    if (this.isSaving) return
    this.isOpen = false
    this.reset()
  }

  public setOpen = (isOpen: boolean) => {
    if (isOpen) {
      this.open()
      return
    }

    this.close()
  }

  public setProvider(value: Provider) {
    this.employee.provider = value
    this.employee.slug = ''
  }

  public setSlug(value: string) {
    this.employee.slug = value
  }

  public setSkillAt(index: number, skill: Skill | null) {
    const nextSkills = [...this.employee.skill_stack]
    if (skill) {
      nextSkills[index] = { name: skill.name, path: skill.path }
    } else {
      nextSkills.splice(index, 1)
    }

    this.employee.skill_stack = nextSkills
  }

  public save = async () => {
    if (!this.employee.isIdentityValid || this.isSaving) return null
    if (this.employee.id) return null

    this.isSaving = true

    try {
      const employee = await employeesApi.create(this.employee.toPayload())
      await this.onCreated?.(employee)

      runInAction(() => {
        this.isOpen = false
        this.reset()
      })

      return employee
    } finally {
      runInAction(() => {
        this.isSaving = false
      })
    }
  }

  private reset = () => {
    this.employee = new EmployeeModel()
  }

  private async loadSkills() {
    if (this.isSkillsLoading || this.availableSkills.length > 0) return

    runInAction(() => {
      this.isSkillsLoading = true
      this.skillsErrorMessage = null
    })

    try {
      const availableSkills = await skillsApi.list()
      runInAction(() => {
        this.availableSkills = availableSkills
      })
    } catch (error) {
      runInAction(() => {
        this.skillsErrorMessage = error instanceof Error ? error.message : 'Unable to load available skills.'
      })
    } finally {
      runInAction(() => {
        this.isSkillsLoading = false
      })
    }
  }
}
