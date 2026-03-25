import { makeAutoObservable } from 'mobx'
import { createContext, useContext } from 'react'
import { employeesApi } from '@/lib/api/employees'
import { issuesApi } from '@/lib/api/issues'
import { projectsApi } from '@/lib/api/projects'
import { AppModel } from '@/models/app.model'
import { EmployeeModel } from '@/models/employee.model'
import { IssueModel } from '@/models/issue.model'
import { ProjectModel } from '@/models/project.model'

export enum OnboardingStep {
  Owner,
  Provider,
  Project,
  Issue,
}

export class OnboardingViewmodel {
  public step: OnboardingStep = OnboardingStep.Owner
  public owner = new EmployeeModel()
  public project = new ProjectModel()
  public issue = new IssueModel()

  constructor() {
    makeAutoObservable(this)
  }

  public init() {
    if (!AppModel.instance.owner) this.setStep(OnboardingStep.Owner)
    else if (!AppModel.instance.hasProvider) this.setStep(OnboardingStep.Provider)
    else if (!AppModel.instance.hasProjects) this.setStep(OnboardingStep.Project)
    else if (!AppModel.instance.hasIssues) this.setStep(OnboardingStep.Issue)
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
    await projectsApi.create(this.project.toPayload())
    AppModel.instance.setHasProjects(true)
    this.setStep(OnboardingStep.Issue)
  }

  public saveIssue = async () => {
    await issuesApi.create(this.issue.toPayload())
    AppModel.instance.setHasIssues(true)
  }
}

export const OnboardingViewmodelContext = createContext<OnboardingViewmodel | null>(null)
export const useOnboardingViewmodel = () => {
  const onboardingViewmodel = useContext(OnboardingViewmodelContext)
  if (!onboardingViewmodel) throw new Error('OnboardingViewmodel not found')

  return onboardingViewmodel
}
