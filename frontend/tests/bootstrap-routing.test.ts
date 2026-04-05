import assert from 'node:assert/strict'
import { test } from 'vitest'

import { getBootstrapRedirectPath, shouldRenderProductShell } from '../src/lib/bootstrap-routing.ts'

test('bootstrap routing redirects first-time users into onboarding', () => {
  assert.equal(getBootstrapRedirectPath({ isAuthenticated: true, isOnboarded: false, pathname: '/' }), '/onboarding')
})

test('bootstrap routing redirects unauthenticated owners without login setup into bootstrap', () => {
  assert.equal(
    getBootstrapRedirectPath({
      hasOwner: true,
      isAuthenticated: false,
      isLoginConfigured: false,
      isOnboarded: false,
      pathname: '/login',
    }),
    '/bootstrap',
  )
})

test('bootstrap routing redirects unauthenticated owners with login setup into login', () => {
  assert.equal(
    getBootstrapRedirectPath({
      hasOwner: true,
      isAuthenticated: false,
      isLoginConfigured: true,
      isOnboarded: false,
      pathname: '/bootstrap',
    }),
    '/login',
  )
})

test('bootstrap routing redirects onboarded users away from onboarding', () => {
  assert.equal(getBootstrapRedirectPath({ isAuthenticated: true, isOnboarded: true, pathname: '/onboarding' }), '/')
})

test('product shell stays disabled on the onboarding route only', () => {
  assert.equal(shouldRenderProductShell('/onboarding'), false)
  assert.equal(shouldRenderProductShell('/'), true)
})
