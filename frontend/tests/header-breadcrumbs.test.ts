import assert from 'node:assert/strict'
import { test } from 'vitest'

import { buildBreadcrumbs } from '../src/lib/router/breadcrumbs.ts'

test('buildBreadcrumbs prefers dynamic overrides for the current route label', () => {
  const crumbs = buildBreadcrumbs({
    currentParams: {
      issueId: 'issue-42',
    },
    currentRouteId: '/issues/$issueId/',
    getOverrideLabel: (routeId) => (routeId === '/issues/$issueId/' ? 'Build issue detail page' : undefined),
    routesById: {
      '/issues/': {
        options: {
          staticData: {
            breadcrumb: 'Issues',
          },
        },
        path: 'issues',
        to: '/issues',
      },
      '/issues/$issueId/': {
        options: {
          staticData: {
            breadcrumb: ({ issueId }: Record<string, string>) => `Issue ${issueId.slice(0, 8)}`,
          },
        },
        path: '$issueId',
        to: '/issues/$issueId',
      },
    },
  })

  assert.deepEqual(crumbs, [
    {
      href: '/issues',
      label: 'Issues',
    },
    {
      href: '/issues/$issueId',
      label: 'Build issue detail page',
    },
  ])
})

test('buildBreadcrumbs falls back to static route metadata when no override exists', () => {
  const crumbs = buildBreadcrumbs({
    currentParams: {
      issueId: 'issue-42',
    },
    currentRouteId: '/issues/$issueId/',
    getOverrideLabel: () => undefined,
    routesById: {
      '/issues/': {
        options: {
          staticData: {
            breadcrumb: 'Issues',
          },
        },
        path: 'issues',
        to: '/issues',
      },
      '/issues/$issueId/': {
        options: {
          staticData: {
            breadcrumb: ({ issueId }: Record<string, string>) => `Issue ${issueId.slice(0, 8)}`,
          },
        },
        path: '$issueId',
        to: '/issues/$issueId',
      },
    },
  })

  assert.deepEqual(crumbs, [
    {
      href: '/issues',
      label: 'Issues',
    },
    {
      href: '/issues/$issueId',
      label: 'Issue issue-42',
    },
  ])
})
