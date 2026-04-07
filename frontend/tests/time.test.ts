import { describe, expect, it } from 'vitest'
import { formatRelativeTime } from '../src/lib/time'

describe('formatRelativeTime', () => {
  it('formats recent timestamps relative to a provided reference time', () => {
    const now = new Date('2026-04-06T16:45:00.000Z')

    expect(formatRelativeTime(new Date('2026-04-06T16:44:45.000Z'), now)).toBe('a few seconds ago')
    expect(formatRelativeTime(new Date('2026-04-06T15:45:00.000Z'), now)).toBe('an hour ago')
  })

  it('returns Unknown for invalid dates', () => {
    expect(formatRelativeTime(new Date('invalid'))).toBe('Unknown')
  })
})