import assert from 'node:assert/strict'
import { test } from 'vitest'

import type { ProviderDto } from '../src/bindings/ProviderDto.ts'
import { ProvidersViewmodel } from '../src/components/pages/providers/providers.viewmodel.ts'
import { providersApi } from '../src/lib/api/providers.ts'

const codexProvider: ProviderDto = {
  base_url: null,
  created_at: '2026-04-01T10:00:00.000Z',
  id: 'provider-codex',
  provider: 'codex',
  updated_at: '2026-04-01T10:00:00.000Z',
}

const openAiProvider: ProviderDto = {
  base_url: 'https://api.openai.com/v1',
  created_at: '2026-04-01T12:00:00.000Z',
  id: 'provider-openai',
  provider: 'openai',
  updated_at: '2026-04-01T12:00:00.000Z',
}

test('ProvidersViewmodel.init loads configured providers and keeps the remaining catalog available', async (t) => {
  const originalList = providersApi.list

  t.onTestFinished(() => {
    providersApi.list = originalList
  })

  providersApi.list = async () => [codexProvider]

  const viewmodel = new ProvidersViewmodel()

  await viewmodel.init()

  assert.equal(viewmodel.connectedProviders.length, 1)
  assert.equal(viewmodel.connectedProviders[0]?.provider, 'codex')
  assert.equal(viewmodel.findProvider('codex')?.id, codexProvider.id)
  assert.equal(
    viewmodel.availableOptions.some((option) => option.provider === 'codex'),
    false,
  )
  assert.equal(
    viewmodel.availableOptions.some((option) => option.provider === 'openai'),
    true,
  )
})

test('saving from the provider sheet adds a newly connected provider into the page state', async (t) => {
  const originalCreate = providersApi.create

  t.onTestFinished(() => {
    providersApi.create = originalCreate
  })

  providersApi.create = async (payload) => ({
    ...openAiProvider,
    base_url: payload.base_url ?? null,
    provider: payload.provider,
  })

  const viewmodel = new ProvidersViewmodel()

  viewmodel.sheet.openForCreate('openai')
  viewmodel.sheet.editor.provider.apiKey = 'sk-test'
  viewmodel.sheet.editor.provider.baseUrl = 'https://api.openai.com/v1'

  const provider = await viewmodel.sheet.save()

  assert.equal(provider?.id, openAiProvider.id)
  assert.equal(viewmodel.findProvider('openai')?.id, openAiProvider.id)
  assert.equal(viewmodel.connectedProviders.length, 1)
  assert.equal(viewmodel.sheet.isOpen, false)
})

test('deleteProvider removes the disconnected provider from the page state', async (t) => {
  const originalDelete = providersApi.delete
  const originalList = providersApi.list

  t.onTestFinished(() => {
    providersApi.delete = originalDelete
    providersApi.list = originalList
  })

  providersApi.list = async () => [codexProvider, openAiProvider]
  providersApi.delete = async () => {}

  const viewmodel = new ProvidersViewmodel()

  await viewmodel.init()
  await viewmodel.deleteProvider(codexProvider.id)

  assert.equal(viewmodel.findProvider('codex'), null)
  assert.deepEqual(
    viewmodel.connectedProviders.map((provider) => provider.provider),
    ['openai'],
  )
})
