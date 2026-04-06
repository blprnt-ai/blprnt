import assert from 'node:assert/strict'
import { test } from 'vitest'

import type { Employee } from '../src/bindings/Employee.ts'
import type { IssueDto } from '../src/bindings/IssueDto.ts'
import { IssuesViewModel } from '../src/components/pages/issues/issues.viewmodel.ts'
import { employeesApi } from '../src/lib/api/employees.ts'
import { issuesApi } from '../src/lib/api/issues.ts'

const employeeFixture: Employee = {
  capabilities: [],
  chain_of_command: [],
  color: 'blue',
  created_at: '2026-04-06T14:00:00.000Z',
  icon: 'dog',
  id: 'employee-1',
  kind: 'agent',
  name: 'Frontend Engineer',
  permissions: null,
  provider_config: null,
  reports_to: null,
  role: 'staff',
  runtime_config: null,
  status: 'running',
  title: 'Frontend Engineer',
}

const issueFixture = (id: string, overrides: Partial<IssueDto> = {}): IssueDto => ({
  actions: [],
  assignee: null,
  attachments: [],
  blocked_by: null,
  checked_out_by: null,
  comments: [],
  created_at: '2026-04-06T14:00:00.000Z',
  creator: 'owner-1',
  description: 'Issue description',
  id,
  identifier: `ISSUE-${id}`,
  labels: [],
  parent_id: null,
  priority: 'medium',
  project: null,
  status: 'todo',
  title: `Issue ${id}`,
  updated_at: '2026-04-06T14:00:00.000Z',
  ...overrides,
})

test('archiveSelectedIssues archives each selected issue and clears selection', async (t) => {
  const originalListIssues = issuesApi.list
  const originalListEmployees = employeesApi.list
  const originalUpdate = issuesApi.update

  t.onTestFinished(() => {
    issuesApi.list = originalListIssues
    employeesApi.list = originalListEmployees
    issuesApi.update = originalUpdate
  })

  const issues = [issueFixture('1'), issueFixture('2')]
  const updates: Array<{ id: string; status?: string }> = []

  issuesApi.list = async () => issues
  employeesApi.list = async () => [employeeFixture]
  issuesApi.update = async (id, data) => {
    updates.push({ id, status: data.status })
    return issueFixture(id, { status: (data.status as IssueDto['status']) ?? 'todo' })
  }

  const viewmodel = new IssuesViewModel('employee-1')
  await viewmodel.init()

  viewmodel.toggleIssueSelection('1')
  viewmodel.toggleIssueSelection('2')
  await viewmodel.archiveSelectedIssues()

  assert.deepEqual(
    updates.map((update) => [update.id, update.status]),
    [
      ['1', 'archived'],
      ['2', 'archived'],
    ],
  )
  assert.equal(viewmodel.selectedIssueIds.size, 0)
})