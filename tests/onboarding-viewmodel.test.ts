import assert from 'node:assert/strict'
import test from 'node:test'

import type { Employee } from '../src/bindings/Employee.ts'
import type { IssueDto } from '../src/bindings/IssueDto.ts'
import type { ProjectDto } from '../src/bindings/ProjectDto.ts'
import { employeesApi } from '../src/lib/api/employees.ts'
import { issuesApi } from '../src/lib/api/issues.ts'
import { projectsApi } from '../src/lib/api/projects.ts'
import { AppModel } from '../src/models/app.model.ts'
import { OnboardingStep, OnboardingViewmodel } from '../src/components/pages/onboarding/onboarding.viewmodel.ts'

const createdCeo: Employee = {
  id: 'ceo-123',
  name: 'Ada Lovelace',
  role: 'ceo',
  kind: 'person',
  icon: 'briefcase',
  color: 'blue',
  title: 'Chief Executive Officer',
  status: 'running',
  capabilities: [],
  permissions: null,
  reports_to: null,
  provider_config: null,
  runtime_config: null,
  chain_of_command: [],
}

const createdProject: ProjectDto = {
  id: 'project-123',
  name: 'Launchpad',
  working_directories: ['/Users/test/projects/launchpad'],
  created_at: '2026-03-25T00:00:00.000Z',
  updated_at: '2026-03-25T00:00:00.000Z',
}

const createdIssue: IssueDto = {
  id: 'issue-123',
  identifier: 'BLP-1',
  title: 'Kick off company setup',
  description: 'Start here',
  status: 'Todo',
  project: createdProject.id,
  parent_id: null,
  creator: null,
  assignee: createdCeo.id,
  blocked_by: null,
  checked_out_by: null,
  priority: 'Medium',
  created_at: '2026-03-25T00:00:00.000Z',
  updated_at: '2026-03-25T00:00:00.000Z',
  comments: [],
  attachments: [],
  actions: [],
}

const resetAppModel = () => {
  AppModel.instance.owner = null
  AppModel.instance.setHasProvider(false)
  AppModel.instance.setHasProjects(false)
  AppModel.instance.setHasIssues(false)
}

test('saveProject advances onboarding into the CEO step and carries the created project id into the issue model', async (t) => {
  resetAppModel()

  const originalCreate = projectsApi.create
  let payload: ProjectDto | null = null

  t.after(() => {
    projectsApi.create = originalCreate
  })

  projectsApi.create = async (data) => {
    payload = {
      id: createdProject.id,
      name: data.name,
      working_directories: data.working_directories,
      created_at: createdProject.created_at,
      updated_at: createdProject.updated_at,
    }

    return payload
  }

  const viewmodel = new OnboardingViewmodel()
  viewmodel.step = OnboardingStep.Project
  viewmodel.project.name = createdProject.name
  viewmodel.project.workingDirectories.push(...createdProject.working_directories)

  await viewmodel.saveProject()

  assert.equal(payload?.name, createdProject.name)
  assert.equal(viewmodel.step, OnboardingStep.Ceo)
  assert.equal(viewmodel.issue.project, createdProject.id)
  assert.equal(AppModel.instance.hasProjects, true)
})

test('saveCeo creates a person CEO with onboarding defaults and preassigns the first issue', async (t) => {
  resetAppModel()

  const originalCreate = employeesApi.create

  t.after(() => {
    employeesApi.create = originalCreate
  })

  let payload: Parameters<typeof employeesApi.create>[0] | null = null

  employeesApi.create = async (data) => {
    payload = data
    return createdCeo
  }

  const viewmodel = new OnboardingViewmodel()
  viewmodel.step = OnboardingStep.Ceo
  viewmodel.ceo.name = createdCeo.name
  viewmodel.ceo.icon = createdCeo.icon
  viewmodel.ceo.color = createdCeo.color

  await viewmodel.saveCeo()

  assert.deepEqual(payload, {
    capabilities: [],
    color: createdCeo.color,
    icon: createdCeo.icon,
    kind: 'person',
    name: createdCeo.name,
    provider_config: null,
    role: 'ceo',
    runtime_config: null,
    title: 'Chief Executive Officer',
  })
  assert.equal(viewmodel.step, OnboardingStep.Issue)
  assert.equal(viewmodel.issue.assignee, createdCeo.id)
})

test('saveIssue submits the first issue using the created project and CEO ids', async (t) => {
  resetAppModel()

  const originalCreate = issuesApi.create

  t.after(() => {
    issuesApi.create = originalCreate
  })

  let payload: Parameters<typeof issuesApi.create>[0] | null = null

  issuesApi.create = async (data) => {
    payload = data
    return createdIssue
  }

  const viewmodel = new OnboardingViewmodel()
  viewmodel.issue.title = createdIssue.title
  viewmodel.issue.description = createdIssue.description
  viewmodel.issue.project = createdProject.id
  viewmodel.issue.assignee = createdCeo.id

  await viewmodel.saveIssue()

  assert.deepEqual(payload, {
    assignee: createdCeo.id,
    description: createdIssue.description,
    parent: null,
    priority: 'Medium',
    project: createdProject.id,
    title: createdIssue.title,
  })
  assert.equal(AppModel.instance.hasIssues, true)
})
