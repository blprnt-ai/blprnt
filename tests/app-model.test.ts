import assert from 'node:assert/strict'
import test from 'node:test'

import type { Employee } from '../src/bindings/Employee'
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
