import assert from 'node:assert/strict'
import test from 'node:test'

import type { Employee } from '../src/bindings/Employee'
import type { ProjectDto } from '../src/bindings/ProjectDto'
import { apiClient } from '../src/lib/api/fetch.ts'
import { AppModel } from '../src/models/app.model.ts'

class LocalStorageStub {
  private store = new Map<string, string>()

  public getItem(key: string): string | null {
    return this.store.get(key) ?? null
  }

  public setItem(key: string, value: string): void {
    this.store.set(key, value)
  }

  public removeItem(key: string): void {
    this.store.delete(key)
  }
}

const owner: Employee = {
  id: 'owner-123',
  name: 'Owner',
  role: 'owner',
  kind: 'person',
  icon: 'briefcase',
  color: 'blue',
  title: 'Owner',
  status: 'running',
  capabilities: [],
  permissions: null,
  reports_to: null,
  provider_config: null,
  runtime_config: null,
  chain_of_command: [],
}

const ceo: Employee = {
  id: 'ceo-123',
  name: 'Ada Lovelace',
  role: 'ceo',
  kind: 'person',
  icon: 'brain',
  color: 'purple',
  title: 'Chief Executive Officer',
  status: 'running',
  capabilities: [],
  permissions: null,
  reports_to: owner.id,
  provider_config: null,
  runtime_config: null,
  chain_of_command: [],
}

const project: ProjectDto = {
  id: 'project-123',
  name: 'Launchpad',
  working_directories: ['/tmp/launchpad'],
  created_at: '2026-03-26T10:00:00.000Z',
  updated_at: '2026-03-26T10:00:00.000Z',
}

test('AppModel.setOwner keeps the active owner in memory for API calls without marking onboarding complete', () => {
  globalThis.localStorage = new LocalStorageStub() as Storage
  apiClient.setEmployeeId(null)

  const model = new (AppModel as unknown as new () => AppModel)()

  model.setOwner(owner)

  assert.equal(model.owner?.id, owner.id)
  assert.equal(model.isOnboarded, false)
  assert.equal(apiClient.employeeId, owner.id)
  assert.equal(globalThis.localStorage.getItem('ownerId'), null)
  assert.equal(globalThis.localStorage.getItem('isOnboarded'), null)
})

test('AppModel stores employees and resolves ids to employee names', () => {
  globalThis.localStorage = new LocalStorageStub() as Storage

  const model = new (AppModel as unknown as new () => AppModel)()

  model.setEmployees([owner, ceo])

  assert.equal(model.employees.length, 2)
  assert.equal(model.resolveEmployeeName(ceo.id), ceo.name)
  assert.equal(model.resolveEmployeeName('missing-employee'), 'missing-employee')
  assert.equal(model.resolveEmployeeName(null), null)
})

test('AppModel stores projects and resolves ids to project names', () => {
  globalThis.localStorage = new LocalStorageStub() as Storage

  const model = new (AppModel as unknown as new () => AppModel)()

  model.setProjects([project])

  assert.equal(model.projects.length, 1)
  assert.equal(model.resolveProjectName(project.id), project.name)
  assert.equal(model.resolveProjectName('missing-project'), 'missing-project')
  assert.equal(model.resolveProjectName(null), null)
})
