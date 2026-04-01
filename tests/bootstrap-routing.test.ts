import assert from 'node:assert/strict'
import { test } from 'vitest'

import { getBootstrapRedirectPath, shouldRenderProductShell } from '../src/lib/bootstrap-routing.ts'

test('bootstrap routing redirects first-time users into onboarding', () => {
  assert.equal(getBootstrapRedirectPath({ isOnboarded: false, pathname: '/' }), '/onboarding')
})

test('bootstrap routing redirects onboarded users away from onboarding', () => {
  assert.equal(getBootstrapRedirectPath({ isOnboarded: true, pathname: '/onboarding' }), '/')
})

test('product shell stays disabled on the onboarding route only', () => {
  assert.equal(shouldRenderProductShell('/onboarding'), false)
  assert.equal(shouldRenderProductShell('/'), true)
})
