import { makeAutoObservable, runInAction } from 'mobx'
import { createContext, useContext } from 'react'
import type { BootstrapOwnerPayload } from '@/bindings/BootstrapOwnerPayload'
import type { Employee } from '@/bindings/Employee'
import type { ProjectDto } from '@/bindings/ProjectDto'
import type { ProviderDto } from '@/bindings/ProviderDto'
import type { Skill } from '@/bindings/Skill'
import { employeesApi } from '@/lib/api/employees'
import { issuesApi } from '@/lib/api/issues'
import { projectsApi } from '@/lib/api/projects'
import { providersApi } from '@/lib/api/providers'
import { skillsApi } from '@/lib/api/skills'
import { AppModel } from '@/models/app.model'
import { EmployeeModel } from '@/models/employee.model'
import { IssueModel } from '@/models/issue.model'
import { ProjectModel } from '@/models/project.model'
import { ProviderModel } from '@/models/provider.model'

export enum OnboardingStep {
  Owner,
  Provider,
  Project,
  Ceo,
  Issue,
  Done,
}

export class OnboardingViewmodel {
  public availableSkills: Skill[] = []
  public isSkillsLoading = false
  public skillsErrorMessage: string | null = null
  public step: OnboardingStep = OnboardingStep.Owner
  public owner = new EmployeeModel()
  public provider: ProviderModel = new ProviderModel()
  public ceo = this.createCeoModel()
  public project = new ProjectModel()
  public issue = this.createIssueModel()

  constructor() {
    makeAutoObservable(
      this,
      {},
      {
        autoBind: true,
      },
    )
  }

  public async init() {
    const owner = await employeesApi.getOwner()
    if (owner) this.setOwner(owner)
    else return

    const providers = await providersApi.list()
    if (providers.length > 0) this.setProvider(providers[0])

    const projects = await projectsApi.list()
    if (projects.length > 0) this.setProject(projects[0])

    const employees = await employeesApi.list()
    AppModel.instance.setEmployees(employees)
    if (employees.length > 1) {
      const ceoEmployee = employees.find((employee) => employee.role === 'ceo')
      if (ceoEmployee) {
        this.setCeo(ceoEmployee)
        this.ceo.setProvider(this.provider.provider)
        this.ceo.clearDirty()
      }
    }

    const issues = await issuesApi.list()
    if (issues.length) {
      AppModel.instance.setIsOnboarded(true)
      this.setStep(OnboardingStep.Done)
      return
    }

    this.issue.setAssignee(this.ceo.id!)
    this.issue.setProject(this.project.id!)

    if (!this.owner.id) this.setStep(OnboardingStep.Owner)
    else if (!this.provider.id) this.setStep(OnboardingStep.Provider)
    else if (!this.project.id) this.setStep(OnboardingStep.Project)
    else if (!this.ceo.id) this.setStep(OnboardingStep.Ceo)
    else this.setStep(OnboardingStep.Issue)
  }

  public setStep(step: OnboardingStep) {
    this.step = step
  }

  public saveOwner = async () => {
    const owner = this.owner.id ? await this.updateOwner() : await this.createOwner()
    this.setOwner(owner)
    this.setStep(OnboardingStep.Provider)
  }

  private createOwner = async () => {
    return employeesApi.ownerOnboarding(this.owner.toOwnerOnboardingPayload() as BootstrapOwnerPayload)
  }

  private updateOwner = async () => {
    return employeesApi.update(this.owner.id!, this.owner.toPayloadPatch())
  }

  public setOwner = (owner: Employee) => {
    this.owner = new EmployeeModel(owner)
    AppModel.instance.setOwner(owner)
    this.setStep(OnboardingStep.Provider)
  }

  public setProvider = (provider: ProviderDto) => {
    this.provider = new ProviderModel(provider)
    this.ceo.setProvider(provider.provider)
    this.setStep(OnboardingStep.Project)
  }

  public saveProject = async () => {
    const project = this.project.id ? await this.updateProject() : await this.createProject()
    this.issue.project = project.id
    this.setProject(project)
  }

  private createProject = async () => {
    return projectsApi.create(this.project.toPayload())
  }

  private updateProject = async () => {
    return projectsApi.update(this.project.id!, this.project.toPayloadPatch())
  }

  public setProject = (project: ProjectDto) => {
    this.project = new ProjectModel(project)
    AppModel.instance.upsertProject(project)
    this.setStep(OnboardingStep.Ceo)
  }

  public saveCeo = async () => {
    const ceo = this.ceo.id ? await this.updateCeo() : await this.createCeo()
    this.issue.assignee = ceo.id
    this.setCeo(ceo)
  }

  public async ensureSkillsLoaded() {
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

  public setCeoSkillAt(index: number, skill: Skill | null) {
    const nextSkills = [...this.ceo.skill_stack]
    if (skill) {
      nextSkills[index] = { name: skill.name, path: skill.path }
    } else {
      nextSkills.splice(index, 1)
    }

    this.ceo.skill_stack = nextSkills
  }

  private createCeo = async () => {
    return employeesApi.create(this.ceo.toPayload())
  }

  private updateCeo = async () => {
    return employeesApi.update(this.ceo.id!, this.ceo.toPayloadPatch())
  }

  public setCeo = (ceo: Employee) => {
    this.ceo = new EmployeeModel(ceo)
    AppModel.instance.upsertEmployee(ceo)
    this.setStep(OnboardingStep.Issue)
  }

  public saveIssue = async () => {
    await issuesApi.create(this.issue.toPayload())
    AppModel.instance.setIsOnboarded(true)

    this.setStep(OnboardingStep.Done)
  }

  private createCeoModel() {
    const ceo = new EmployeeModel()
    ceo.name = 'CEO'
    ceo.role = 'ceo'
    ceo.title = 'Chief Executive Officer'

    return ceo
  }

  private createIssueModel() {
    const issue = new IssueModel()
    issue.title = 'Create your CEO HEARTBEAT.md'
    issue.status = 'todo'
    issue.description = `Setup yourself as the CEO. Use the ceo persona found here:

https://github.com/blprnt-ai/employees/blob/main/employees/ceo/AGENTS.md

Save this AGENTS.md and the sibling HEARTBEAT.md, SOUL.md, and TOOLS.md in $AGENT_HOME.

After that, hire yourself a CTO agent and then plan the roadmap and tasks for your new company.`

    return issue
  }
}

export const OnboardingViewmodelContext = createContext<OnboardingViewmodel | null>(null)
export const useOnboardingViewmodel = () => {
  const onboardingViewmodel = useContext(OnboardingViewmodelContext)
  if (!onboardingViewmodel) throw new Error('OnboardingViewmodel not found')

  return onboardingViewmodel
}
