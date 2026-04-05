import assert from 'node:assert/strict'
import { test } from 'vitest'

import type { IssueDto } from '../src/bindings/IssueDto.ts'
import { IssueFormViewmodel } from '../src/components/forms/issue/issue-form.viewmodel.tsx'
import { issuesApi } from '../src/lib/api/issues.ts'

const issueFixture: IssueDto = {
  actions: [],
  assignee: 'employee-1',
  attachments: [],
  blocked_by: null,
  checked_out_by: null,
  comments: [],
  created_at: '2026-03-28T10:00:00.000Z',
  creator: 'owner-1',
  description: 'Outline the initial priorities for the team.',
  id: 'issue-100',
  identifier: 'BLP-100',
  parent_id: null,
  priority: 'high',
  project: 'project-1',
  status: 'todo',
  title: 'Kick off roadmap planning',
  updated_at: '2026-03-28T10:00:00.000Z',
}

test('IssueFormViewmodel.cancel closes and resets the draft', () => {
  const viewmodel = new IssueFormViewmodel()

  viewmodel.open()
  viewmodel.issue.title = 'Temporary issue'
  viewmodel.issue.description = 'Temporary description'

  viewmodel.cancel()

  assert.equal(viewmodel.isOpen, false)
  assert.equal(viewmodel.issue.title, '')
  assert.equal(viewmodel.issue.description, '')
  assert.equal(viewmodel.issue.priority, 'medium')
})

test('IssueFormViewmodel.dismiss closes without resetting the draft', () => {
  const viewmodel = new IssueFormViewmodel()

  viewmodel.open()
  viewmodel.issue.title = 'Temporary issue'
  viewmodel.issue.description = 'Temporary description'

  viewmodel.dismiss()

  assert.equal(viewmodel.isOpen, false)
  assert.equal(viewmodel.issue.title, 'Temporary issue')
  assert.equal(viewmodel.issue.description, 'Temporary description')
})

test('IssueFormViewmodel.open applies optional defaults before showing the sheet', () => {
  const viewmodel = new IssueFormViewmodel()

  viewmodel.openWithDefaults({ assignee: 'employee-1', project: 'project-1' })

  assert.equal(viewmodel.isOpen, true)
  assert.equal(viewmodel.issue.assignee, 'employee-1')
  assert.equal(viewmodel.issue.project, 'project-1')
})

test('IssueFormViewmodel.openWithDefaults preserves an existing draft', () => {
  const viewmodel = new IssueFormViewmodel()

  viewmodel.openWithDefaults({ assignee: 'employee-1', project: 'project-1' })
  viewmodel.issue.title = 'Existing draft'
  viewmodel.issue.assignee = 'employee-2'
  viewmodel.issue.project = 'project-2'

  viewmodel.dismiss()
  viewmodel.openWithDefaults({ assignee: 'employee-3', project: 'project-3' })

  assert.equal(viewmodel.issue.title, 'Existing draft')
  assert.equal(viewmodel.issue.assignee, 'employee-2')
  assert.equal(viewmodel.issue.project, 'project-2')
})

test('IssueFormViewmodel.save creates an issue, closes the sheet, and notifies listeners', async (t) => {
  const originalCreate = issuesApi.create

  t.onTestFinished(() => {
    issuesApi.create = originalCreate
  })

  let payload: Parameters<typeof issuesApi.create>[0] | null = null
  let _createdIssue: IssueDto | null = null

  issuesApi.create = async (data) => {
    payload = data
    return issueFixture
  }

  const viewmodel = new IssueFormViewmodel(async (issue) => {
    _createdIssue = issue
  })

  viewmodel.open()
  viewmodel.issue.title = '  Kick off roadmap planning  '
  viewmodel.issue.description = '  Outline the initial priorities for the team.  '
  viewmodel.issue.project = 'project-1'
  viewmodel.issue.assignee = 'employee-1'
  viewmodel.issue.priority = 'high'

  const savedIssue = await viewmodel.save()

  assert.equal(payload?.title, '  Kick off roadmap planning  ')
  assert.equal(payload?.description, '  Outline the initial priorities for the team.  ')
  assert.equal(payload?.project, 'project-1')
  assert.equal(payload?.assignee, 'employee-1')
  assert.equal(payload?.priority, 'high')
  assert.equal(savedIssue?.id, issueFixture.id)
  assert.equal(viewmodel.isOpen, false)
  assert.equal(viewmodel.issue.title, '')
  assert.equal(viewmodel.issue.description, '')
})
