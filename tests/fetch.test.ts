import assert from 'node:assert/strict'
import { test } from 'vitest'

import { apiClient } from '../src/lib/api/fetch.ts'
import { projectsApi } from '../src/lib/api/projects.ts'

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

  public clear(): void {
    this.store.clear()
  }
}

test('apiClient.delete returns undefined for 204 responses', async () => {
  const localStorage = new LocalStorageStub()
  let request: { url: string; init?: RequestInit } | null = null

  globalThis.localStorage = localStorage as Storage
  globalThis.fetch = (async (url, init) => {
    request = {
      url: String(url),
      init,
    }

    return new Response(null, {
      status: 204,
    })
  }) as typeof fetch

  apiClient.setEmployeeId('employee-123')

  const response = await apiClient.delete<void>('/runs/run-123/cancel')

  assert.equal(response, undefined)
  assert.deepEqual(request, {
    url: 'http://localhost:9171/api/v1/runs/run-123/cancel',
    init: {
      headers: {
        'x-blprnt-employee-id': 'employee-123',
      },
      method: 'DELETE',
    },
  })
})

test('projectsApi.nukeDatabase targets the dev database endpoint', async () => {
  const localStorage = new LocalStorageStub()
  let request: { url: string; init?: RequestInit } | null = null

  globalThis.localStorage = localStorage as Storage
  globalThis.fetch = (async (url, init) => {
    request = {
      url: String(url),
      init,
    }

    return new Response(null, {
      status: 204,
    })
  }) as typeof fetch

  apiClient.setEmployeeId('owner-123')

  const response = await projectsApi.nukeDatabase()

  assert.equal(response, undefined)
  assert.deepEqual(request, {
    url: 'http://localhost:9171/api/v1/dev/database',
    init: {
      headers: {
        'x-blprnt-employee-id': 'owner-123',
      },
      method: 'DELETE',
    },
  })
})
