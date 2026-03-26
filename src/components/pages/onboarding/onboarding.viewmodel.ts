import { makeAutoObservable } from 'mobx'
import { createContext, useContext } from 'react'
import { employeesApi } from '@/lib/api/employees'
import { issuesApi } from '@/lib/api/issues'
import { projectsApi } from '@/lib/api/projects'
import { AppModel } from '@/models/app.model'
import { EmployeeModel } from '@/models/employee.model'
import { IssueModel } from '@/models/issue.model'
import { ProjectModel } from '@/models/project.model'

export const OnboardingStep = {
  Ceo: 'ceo',
  Issue: 'issue',
  Owner: 'owner',
  Project: 'project',
  Provider: 'provider',
} as const

export type OnboardingStep = (typeof OnboardingStep)[keyof typeof OnboardingStep]

export class OnboardingViewmodel {
  public step: OnboardingStep = OnboardingStep.Owner
  public owner = new EmployeeModel()
  public ceo = this.createCeoModel()
  public project = new ProjectModel()
  public issue = new IssueModel()

  constructor() {
    makeAutoObservable(this)
  }

  public init() {
    if (!AppModel.instance.owner) this.setStep(OnboardingStep.Owner)
    else if (!AppModel.instance.hasProvider) this.setStep(OnboardingStep.Provider)
    else if (!AppModel.instance.hasProjects) this.setStep(OnboardingStep.Project)
    else if (!AppModel.instance.hasIssues) this.setStep(OnboardingStep.Ceo)
  }

  public setStep(step: OnboardingStep) {
    this.step = step
  }

  public saveOwner = async () => {
    const owner = await employeesApi.ownerOnboarding(this.owner.toOwnerOnboardingPayload())
    AppModel.instance.setOwner(owner)
    this.setStep(OnboardingStep.Provider)
  }

  public saveProvider = async () => {
    AppModel.instance.setHasProvider(true)
    this.setStep(OnboardingStep.Project)
  }

  public saveProject = async () => {
    const project = await projectsApi.create(this.project.toPayload())
    this.project = new ProjectModel(project)
    this.issue.project = project.id
    AppModel.instance.setHasProjects(true)
    this.setStep(OnboardingStep.Ceo)
  }

  public saveCeo = async () => {
    const ceo = await employeesApi.create(this.ceo.toPayload())
    this.ceo = new EmployeeModel(ceo)
    this.issue.assignee = ceo.id
    AppModel.instance.setHasCeo(true)
    this.setStep(OnboardingStep.Issue)
  }

  public saveIssue = async () => {
    const issue = await issuesApi.create(this.issue.toPayload())
    this.issue = new IssueModel(issue)
    AppModel.instance.setHasIssues(true)
  }

  private createCeoModel() {
    const ceo = new EmployeeModel()
    ceo.kind = 'person'
    ceo.role = 'ceo'
    ceo.title = 'Chief Executive Officer'
    ceo.icon = 'briefcase'
    ceo.color = 'blue'

    return ceo
  }
}

export const OnboardingViewmodelContext = createContext<OnboardingViewmodel | null>(null)
export const useOnboardingViewmodel = () => {
  const onboardingViewmodel = useContext(OnboardingViewmodelContext)
  if (!onboardingViewmodel) throw new Error('OnboardingViewmodel not found')

  return onboardingViewmodel
}
