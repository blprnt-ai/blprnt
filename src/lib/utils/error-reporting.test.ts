import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { createReactRootErrorHandler, reportError } from './error-reporting'

describe('error reporting', () => {
  beforeEach(() => {
    vi.spyOn(console, 'error').mockImplementation(() => {})
  })

  afterEach(() => {
    vi.restoreAllMocks()
  })

  it('logs application errors with a stable prefix and context', () => {
    const error = new Error('update failed')

    reportError(error, 'checking for updates')

    expect(console.error).toHaveBeenCalledWith('[blprnt]', 'checking for updates', error)
  })

  it('logs react root errors with phase metadata', () => {
    const error = new Error('render failed')
    const errorInfo = { componentStack: '\n    at App' }

    createReactRootErrorHandler('uncaught')(error, errorInfo)

    expect(console.error).toHaveBeenCalledWith('[blprnt]', 'react root uncaught error', error, errorInfo)
  })
})
