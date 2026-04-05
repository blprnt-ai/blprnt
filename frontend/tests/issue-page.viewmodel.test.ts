import assert from 'node:assert/strict'
import { test } from 'vitest'

import type { IssueAttachment } from '../src/bindings/IssueAttachment.ts'
import type { IssueDto } from '../src/bindings/IssueDto.ts'
import { issuesApi } from '../src/lib/api/issues.ts'
import { IssueViewmodel } from '../src/components/pages/issue/issue.viewmodel.ts'

const issueFixture: IssueDto = {
  id: 'issue-42',
  identifier: 'BLP-42',
  title: 'Build issue detail page',
  description: 'Users need a detailed issue workspace.',
  status: 'in_progress',
  project: 'project-1',
  parent_id: null,
  creator: 'Ada Lovelace',
  assignee: 'Grace Hopper',
  blocked_by: null,
  checked_out_by: 'Grace Hopper',
  priority: 'high',
  created_at: '2026-03-25T10:00:00.000Z',
  updated_at: '2026-03-26T10:00:00.000Z',
  comments: [],
  attachments: [],
  actions: [],
}

const childIssueFixture: IssueDto = {
  ...issueFixture,
  id: 'issue-43',
  identifier: 'BLP-43',
  title: 'Render child issue tab',
  description: 'Show linked child issues in the issue page tabs.',
  parent_id: issueFixture.id,
}

test('init loads child issues alongside the selected issue', async (t) => {
  const originalGet = issuesApi.get
  const originalGetAttachment = issuesApi.getAttachment
  const originalListChildren = issuesApi.listChildren
  const originalListRuns = issuesApi.listRuns

  t.onTestFinished(() => {
    issuesApi.get = originalGet
    issuesApi.getAttachment = originalGetAttachment
    issuesApi.listChildren = originalListChildren
    issuesApi.listRuns = originalListRuns
  })

  issuesApi.get = async () => issueFixture
  issuesApi.getAttachment = async () => {
    throw new Error('attachment hydration should not run when there are no attachments')
  }
  issuesApi.listChildren = async () => [childIssueFixture]
  issuesApi.listRuns = async () => []

  const viewmodel = new IssueViewmodel(issueFixture.id, 'employee-1', async () => {
    throw new Error('file reader should not be used for init tests')
  })

  await viewmodel.init()

  assert.equal(viewmodel.issue?.id, issueFixture.id)
  assert.equal(viewmodel.childIssues.length, 1)
  assert.equal(viewmodel.childIssues[0]?.id, childIssueFixture.id)
  assert.equal(viewmodel.childIssues[0]?.parent, issueFixture.id)
})

test('submitComment trims the draft, persists it, and appends it to the issue', async (t) => {
  const originalGet = issuesApi.get
  const originalGetAttachment = issuesApi.getAttachment
  const originalComment = issuesApi.comment
  const originalListRuns = issuesApi.listRuns

  t.onTestFinished(() => {
    issuesApi.get = originalGet
    issuesApi.getAttachment = originalGetAttachment
    issuesApi.comment = originalComment
    issuesApi.listRuns = originalListRuns
  })

  let payload: Parameters<typeof issuesApi.comment>[1] | null = null

  issuesApi.get = async () => issueFixture
  issuesApi.getAttachment = async () => {
    throw new Error('attachment hydration should not run when there are no attachments')
  }
  issuesApi.comment = async (_id, data) => {
    payload = data

    return {
      id: 'comment-1',
      comment: data.comment,
      mentions: [],
      creator: 'Ada Lovelace',
      run_id: null,
      created_at: '2026-03-26T11:00:00.000Z',
    }
  }
  issuesApi.listRuns = async () => []

  const viewmodel = new IssueViewmodel(issueFixture.id, 'employee-1', async () => {
    throw new Error('file reader should not be used for comment tests')
  })

  await viewmodel.init()
  viewmodel.setCommentDraft('  This page now has discussion.  ')

  await viewmodel.submitComment()

  assert.equal(payload?.comment, 'This page now has discussion.')
  assert.equal(viewmodel.commentDraft, '')
  assert.equal(viewmodel.issue?.comments.length, 1)
  assert.equal(viewmodel.issue?.comments[0]?.comment, 'This page now has discussion.')
})

test('addAttachments converts browser files into issue attachments and appends them', async (t) => {
  const originalGet = issuesApi.get
  const originalGetAttachment = issuesApi.getAttachment
  const originalAttachment = issuesApi.attachment
  const originalListRuns = issuesApi.listRuns

  t.onTestFinished(() => {
    issuesApi.get = originalGet
    issuesApi.getAttachment = originalGetAttachment
    issuesApi.attachment = originalAttachment
    issuesApi.listRuns = originalListRuns
  })

  const payloads: IssueAttachment[] = []

  issuesApi.get = async () => issueFixture
  issuesApi.getAttachment = async () => {
    throw new Error('attachment hydration should not run when there are no attachments')
  }
  issuesApi.listRuns = async () => []
  issuesApi.attachment = async (_id, data) => {
    payloads.push(data)

    return {
      id: `attachment-${payloads.length}`,
      name: data.name,
      attachment_kind: data.attachment_kind,
      mime_kind: data.mime_kind,
      size: data.size,
      run_id: null,
      created_at: '2026-03-26T11:30:00.000Z',
    }
  }

  const viewmodel = new IssueViewmodel(issueFixture.id, 'employee-1', async () => issueFixture.id)

  await viewmodel.init()

  await viewmodel.addAttachments([
    {
      name: 'mockup.png',
      size: 2048,
      type: 'image/png',
    },
    {
      name: 'spec.txt',
      size: 512,
      type: 'text/plain',
    },
  ])

  assert.equal(payloads.length, 2)
  assert.deepEqual(payloads.map((payload) => payload.attachment_kind), ['image', 'file'])
  assert.deepEqual(payloads.map((payload) => payload.attachment), [issueFixture.id, issueFixture.id])
  assert.equal(viewmodel.issue?.attachments.length, 2)
  assert.equal(viewmodel.issue?.attachments[0]?.attachment.name, 'mockup.png')
  assert.equal(viewmodel.issue?.attachments[1]?.attachment.name, 'spec.txt')
})

test('init hydrates each issue attachment after loading issue metadata', async (t) => {
  const originalGet = issuesApi.get
  const originalGetAttachment = issuesApi.getAttachment
  const originalListChildren = issuesApi.listChildren
  const originalListRuns = issuesApi.listRuns

  t.onTestFinished(() => {
    issuesApi.get = originalGet
    issuesApi.getAttachment = originalGetAttachment
    issuesApi.listChildren = originalListChildren
    issuesApi.listRuns = originalListRuns
  })

  const issueWithAttachment: IssueDto = {
    ...issueFixture,
    attachments: [
      {
        id: 'attachment-1',
        name: 'spec.txt',
        attachment_kind: 'file',
        mime_kind: 'text/plain',
        size: 5,
        run_id: null,
        created_at: '2026-03-26T11:30:00.000Z',
      },
    ],
  }

  const attachmentCalls: string[] = []

  issuesApi.get = async () => issueWithAttachment
  issuesApi.getAttachment = async (issueId, attachmentId) => {
    attachmentCalls.push(`${issueId}:${attachmentId}`)

    return {
      id: attachmentId,
      attachment: {
        name: 'spec.txt',
        attachment_kind: 'file',
        attachment: 'data:text/plain;base64,SGVsbG8=',
        mime_kind: 'text/plain',
        size: 5,
      },
      creator: 'Ada Lovelace',
      run_id: null,
      created_at: '2026-03-26T11:30:00.000Z',
    }
  }
  issuesApi.listChildren = async () => []
  issuesApi.listRuns = async () => []

  const viewmodel = new IssueViewmodel(issueFixture.id, 'employee-1', async () => {
    throw new Error('file reader should not be used for hydration tests')
  })

  await viewmodel.init()

  assert.deepEqual(attachmentCalls, [`${issueFixture.id}:attachment-1`])
  assert.equal(viewmodel.issue?.attachments.length, 1)
  assert.equal(viewmodel.issue?.attachments[0]?.attachment.attachment, 'data:text/plain;base64,SGVsbG8=')
  assert.equal(viewmodel.issue?.attachments[0]?.creator, 'Ada Lovelace')
})

test('timelineItems interleave comments and runs chronologically', async (t) => {
  const originalGet = issuesApi.get
  const originalGetAttachment = issuesApi.getAttachment
  const originalListChildren = issuesApi.listChildren
  const originalListRuns = issuesApi.listRuns

  t.onTestFinished(() => {
    issuesApi.get = originalGet
    issuesApi.getAttachment = originalGetAttachment
    issuesApi.listChildren = originalListChildren
    issuesApi.listRuns = originalListRuns
  })

  issuesApi.get = async () => ({
    ...issueFixture,
    comments: [
      {
        id: 'comment-1',
        comment: 'First comment',
        mentions: [],
        creator: 'Ada Lovelace',
        run_id: null,
        created_at: '2026-03-26T11:00:00.000Z',
      },
      {
        id: 'comment-2',
        comment: 'Second comment',
        mentions: [],
        creator: 'Grace Hopper',
        run_id: null,
        created_at: '2026-03-26T13:00:00.000Z',
      },
    ],
  })
  issuesApi.getAttachment = async () => {
    throw new Error('attachment hydration should not run when there are no attachments')
  }
  issuesApi.listChildren = async () => []
  issuesApi.listRuns = async () => [
    {
      id: 'run-1',
      employee_id: 'employee-1',
      status: 'Completed',
      trigger: { issue_assignment: { issue_id: issueFixture.id } },
      usage: null,
      created_at: '2026-03-26T12:00:00.000Z',
      started_at: '2026-03-26T12:01:00.000Z',
      completed_at: '2026-03-26T12:05:00.000Z',
    },
  ]

  const viewmodel = new IssueViewmodel(issueFixture.id, 'employee-1', async () => {
    throw new Error('file reader should not be used for timeline tests')
  })

  await viewmodel.init()

  assert.deepEqual(
    viewmodel.timelineItems.map((item) => item.type === 'comment' ? `comment:${item.comment.id}` : `run:${item.run.id}`),
    ['comment:comment-1', 'run:run-1', 'comment:comment-2'],
  )
})
