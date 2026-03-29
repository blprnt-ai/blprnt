import { makeAutoObservable, runInAction } from 'mobx'
import { createContext, useContext } from 'react'
import type { Employee } from '@/bindings/Employee'
import type { ProjectDto } from '@/bindings/ProjectDto'
import type { ProviderDto } from '@/bindings/ProviderDto'
import { employeesApi } from '@/lib/api/employees'
import { apiClient } from '@/lib/api/fetch'
import { issuesApi } from '@/lib/api/issues'
import { projectsApi } from '@/lib/api/projects'
import { providersApi } from '@/lib/api/providers'
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
  public step: OnboardingStep = OnboardingStep.Owner
  public owner = new EmployeeModel()
  public provider: ProviderModel = new ProviderModel()
  public ceo = this.createCeoModel()
  public project = new ProjectModel()
  public issue = this.createIssueModel()

  constructor() {
    makeAutoObservable(this)
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
      }
    }

    const issues = await issuesApi.list()
    if (issues.length) {
      AppModel.instance.setIsOnboarded(true)
      this.setStep(OnboardingStep.Done)
      return
    }

    this.issue.project = this.project.id!
    this.issue.assignee = this.ceo.id!

    if (!this.owner.id) this.setStep(OnboardingStep.Owner)
    else if (!this.provider.id) this.setStep(OnboardingStep.Provider)
    else if (!this.project.id) this.setStep(OnboardingStep.Project)
    else if (!this.ceo.id) {
      runInAction(() => (this.issue.project = this.project.id!))
      this.setStep(OnboardingStep.Ceo)
    } else {
      runInAction(() => (this.issue.assignee = this.ceo.id!))
      this.setStep(OnboardingStep.Issue)
    }
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
    return employeesApi.ownerOnboarding(this.owner.toOwnerOnboardingPayload())
  }

  private updateOwner = async () => {
    return employeesApi.update(this.owner.id!, this.owner.toPayloadPatch())
  }

  public setOwner = (owner: Employee) => {
    this.owner = new EmployeeModel(owner)
    AppModel.instance.setOwner(owner)
    apiClient.setEmployeeId(owner.id)
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

[LINK TO AGENTS.md]

Use the blprnt API to save this AGENTS.md and the sibling HEARTBEAT.md, SOUL.md, and TOOLS.md.

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
