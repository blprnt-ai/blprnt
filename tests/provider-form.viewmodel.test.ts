import assert from 'node:assert/strict'
import test from 'node:test'

import type { ProviderDto } from '../src/bindings/ProviderDto.ts'
import { providersApi } from '../src/lib/api/providers.ts'
import { ProviderFormViewmodel } from '../src/components/forms/provider/provider.viewmodel.ts'

const createdProvider: ProviderDto = {
  id: 'provider-123',
  provider: 'openai',
  base_url: null,
  created_at: '2026-03-25T00:00:00.000Z',
  updated_at: '2026-03-25T00:00:00.000Z',
}

test('save returns the created provider when persistence succeeds', async (t) => {
  const originalCreate = providersApi.create

  t.after(() => {
    providersApi.create = originalCreate
  })

  providersApi.create = async () => createdProvider

  const viewmodel = new ProviderFormViewmodel()
  viewmodel.provider.provider = createdProvider.provider
  viewmodel.provider.apiKey = 'sk-test'

  const provider = await viewmodel.save()

  assert.deepEqual(provider, createdProvider)
  assert.equal(viewmodel.provider.id, createdProvider.id)
})

test('save returns null when persistence fails', async (t) => {
  const originalCreate = providersApi.create

  t.after(() => {
    providersApi.create = originalCreate
  })

  providersApi.create = async () => {
    throw new Error('network down')
  }

  const viewmodel = new ProviderFormViewmodel()
  viewmodel.provider.provider = 'openai'
  viewmodel.provider.apiKey = 'sk-test'

  const provider = await viewmodel.save()

  assert.equal(provider, null)
  assert.equal(viewmodel.provider.id, '')
  assert.equal(viewmodel.isSaving, false)
})
