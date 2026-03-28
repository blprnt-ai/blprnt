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
  capabilities: [],
  chain_of_command: [],
  color: 'blue',
  icon: 'briefcase',
  id: 'owner-123',
  kind: 'person',
  name: 'Owner',
  permissions: null,
  provider_config: null,
  reports_to: null,
  role: 'owner',
  runtime_config: null,
  status: 'running',
  title: 'Owner',
}

const ceo: Employee = {
  capabilities: [],
  chain_of_command: [],
  color: 'purple',
  icon: 'brain',
  id: 'ceo-123',
  kind: 'person',
  name: 'Ada Lovelace',
  permissions: null,
  provider_config: null,
  reports_to: owner.id,
  role: 'ceo',
  runtime_config: null,
  status: 'running',
  title: 'Chief Executive Officer',
}

const project: ProjectDto = {
  created_at: '2026-03-26T10:00:00.000Z',
  id: 'project-123',
  name: 'Launchpad',
  updated_at: '2026-03-26T10:00:00.000Z',
  working_directories: ['/tmp/launchpad'],
}

test('AppModel.setOwner keeps the active owner in memory for API calls without marking onboarding complete', () => {
  globalThis.localStorage = new LocalStorageStub() as unknown as Storage
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
  globalThis.localStorage = new LocalStorageStub() as unknown as Storage

  const model = new (AppModel as unknown as new () => AppModel)()

  model.setEmployees([owner, ceo])

  assert.equal(model.employees.length, 2)
  assert.equal(model.resolveEmployeeName(ceo.id), ceo.name)
  assert.equal(model.resolveEmployeeName('missing-employee'), 'missing-employee')
  assert.equal(model.resolveEmployeeName(null), null)
})

test('AppModel keeps employees in a deterministic order across set and upsert operations', () => {
  globalThis.localStorage = new LocalStorageStub() as unknown as Storage

  const model = new (AppModel as unknown as new () => AppModel)()
  const zed: Employee = { ...ceo, id: 'zed-1', name: 'Zed Shaw', role: 'staff' }
  const beth: Employee = { ...ceo, id: 'beth-1', name: 'Beth Harmon', role: 'manager' }

  model.setEmployees([zed, owner, beth])

  assert.deepEqual(
    model.employees.map((employee) => employee.name),
    ['Owner', 'Beth Harmon', 'Zed Shaw'],
  )

  model.upsertEmployee({ ...ceo, id: 'aaron-1', name: 'Aaron Swartz', role: 'ceo' })

  assert.deepEqual(
    model.employees.map((employee) => employee.name),
    ['Owner', 'Aaron Swartz', 'Beth Harmon', 'Zed Shaw'],
  )
})

test('AppModel stores projects and resolves ids to project names', () => {
  globalThis.localStorage = new LocalStorageStub() as unknown as Storage

  const model = new (AppModel as unknown as new () => AppModel)()

  model.setProjects([project])

  assert.equal(model.projects.length, 1)
  assert.equal(model.resolveProjectName(project.id), project.name)
  assert.equal(model.resolveProjectName('missing-project'), 'missing-project')
  assert.equal(model.resolveProjectName(null), null)
})

test('AppModel.resetAfterDatabaseNuke clears cached app state and onboarding', () => {
  globalThis.localStorage = new LocalStorageStub() as unknown as Storage
  apiClient.setEmployeeId(null)

  const model = new (AppModel as unknown as new () => AppModel)()

  model.setOwner(owner)
  model.setEmployees([owner, ceo])
  model.setProjects([project])
  model.setIsOnboarded(true)

  model.resetAfterDatabaseNuke()

  assert.equal(model.owner, null)
  assert.deepEqual(model.employees, [])
  assert.deepEqual(model.projects, [])
  assert.equal(model.isOnboarded, false)
  assert.equal(apiClient.employeeId, null)
  assert.equal(globalThis.localStorage.getItem('onboarding-complete'), 'false')
})
