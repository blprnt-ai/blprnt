import assert from 'node:assert/strict'
import { test } from 'vitest'

import type { MinionDto } from '../src/bindings/MinionDto.ts'
import { MinionModel } from '../src/models/minion.model.ts'

const systemMinionFixture: MinionDto = {
  id: 'dreamer',
  source: 'system',
  slug: 'dreamer',
  display_name: 'Dreamer',
  description: 'Built-in dream synthesis minion.',
  enabled: true,
  prompt: null,
  can_edit_definition: false,
  can_toggle_enabled: true,
  created_at: '1970-01-01T00:00:00.000Z',
  updated_at: '1970-01-01T00:00:00.000Z',
}

const customMinionFixture: MinionDto = {
  id: 'custom-1',
  source: 'custom',
  slug: 'note-sweeper',
  display_name: 'Note Sweeper',
  description: 'Cleans up stale notes.',
  enabled: true,
  prompt: 'Keep notes concise.',
  can_edit_definition: true,
  can_toggle_enabled: true,
  created_at: '2026-04-01T12:00:00.000Z',
  updated_at: '2026-04-01T12:00:00.000Z',
}

test('MinionModel builds toggle-only patches for system minions', () => {
  const model = new MinionModel(systemMinionFixture)

  model.enabled = false

  assert.equal(model.isValid, true)
  assert.deepEqual(model.toPayloadPatch(), { enabled: false })
})

test('MinionModel omits unchanged definition fields from custom patches', () => {
  const model = new MinionModel(customMinionFixture)

  model.displayName = 'Note Janitor'

  assert.equal(model.isValid, true)
  assert.deepEqual(model.toPayloadPatch(), { display_name: 'Note Janitor' })
})
