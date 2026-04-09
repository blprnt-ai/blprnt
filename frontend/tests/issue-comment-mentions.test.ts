import { describe, expect, it } from 'vitest'
import type { Employee } from '@/bindings/Employee'
import { formatRunTrigger } from '@/lib/runs'
import {
  filterMentionSuggestions,
  getNextMentionSuggestionIndex,
  getMentionQuery,
  inferMentionSelections,
  insertMentionSelection,
  linkifyIssueIdentifiersInMarkdown,
  linkifyEmployeeMentionsInMarkdown,
  linkifyMentionsInMarkdown,
  mentionPayloadsFromSelections,
  reconcileMentionSelections,
  segmentCommentWithMentions,
} from '@/components/pages/issue/comment-mentions'

const employee = (id: string, name: string): Employee => ({
  id,
  name,
  role: 'employee',
  kind: 'person',
  icon: 'brain',
  color: 'blue',
  title: 'Engineer',
  status: 'idle',
  capabilities: [],
  permissions: null,
  reports_to: null,
  provider_config: null,
  runtime_config: null,
  chain_of_command: [],
  created_at: new Date().toISOString(),
})

describe('issue comment mention helpers', () => {
  it('finds the active mention query and inserts the selected employee', () => {
    const ada = employee('1', 'Ada Lovelace')
    const query = getMentionQuery('Need input from @ad', 'Need input from @ad'.length)

    expect(query).toEqual({ query: 'ad', start: 16, end: 19 })

    const result = insertMentionSelection('Need input from @ad', query!, ada)

    expect(result.nextText).toBe('Need input from @Ada Lovelace ')
    expect(result.selection).toEqual({
      employeeId: '1',
      label: 'Ada Lovelace',
      start: 16,
      end: 29,
    })
  })

  it('filters suggestions by name prefix and word prefix', () => {
    const employees = [employee('1', 'Ada Lovelace'), employee('2', 'Grace Hopper'), employee('3', 'Alan Turing')]

    expect(filterMentionSuggestions(employees, 'hop').map((candidate) => candidate.name)).toEqual(['Grace Hopper'])
    expect(filterMentionSuggestions(employees, 'a').map((candidate) => candidate.name)).toEqual([
      'Ada Lovelace',
      'Alan Turing',
      'Grace Hopper',
    ])
  })

  it('wraps keyboard mention navigation across the suggestion list', () => {
    expect(getNextMentionSuggestionIndex(0, 3, 1)).toBe(1)
    expect(getNextMentionSuggestionIndex(2, 3, 1)).toBe(0)
    expect(getNextMentionSuggestionIndex(0, 3, -1)).toBe(2)
    expect(getNextMentionSuggestionIndex(0, 0, 1)).toBe(0)
  })

  it('formats standard run triggers', () => {
    expect(formatRunTrigger('manual')).toBe('Manual')
  })

  it('reconciles mentions after text edits and drops stale selections', () => {
    const selections = [{ employeeId: '1', label: 'Ada Lovelace', start: 16, end: 29 }]

    expect(reconcileMentionSelections('Please review, @Ada Lovelace today', selections)).toEqual([
      { employeeId: '1', label: 'Ada Lovelace', start: 15, end: 28 },
    ])

    expect(reconcileMentionSelections('Please review today', selections)).toEqual([])
    expect(mentionPayloadsFromSelections([
      { employeeId: '1', label: 'Ada Lovelace', start: 0, end: 13 },
      { employeeId: '1', label: 'Ada Lovelace', start: 20, end: 33 },
    ])).toEqual([{ employee_id: '1', label: 'Ada Lovelace' }])
  })

  it('infers typed mentions without an explicit typeahead selection', () => {
    expect(
      mentionPayloadsFromSelections(
        inferMentionSelections('Please sync with @Ada Lovelace tomorrow.', [employee('1', 'Ada Lovelace')]),
      ),
    ).toEqual([{ employee_id: '1', label: 'Ada Lovelace' }])
  })

  it('does not infer partial or embedded typed mentions', () => {
    expect(inferMentionSelections('Checking with @Ada Love', [employee('1', 'Ada Lovelace')])).toEqual([])
    expect(inferMentionSelections('email@Ada Lovelace', [employee('1', 'Ada Lovelace')])).toEqual([])
  })

  it('preserves selected mentions while inferring additional typed mentions', () => {
    const ada = employee('1', 'Ada Lovelace')
    const grace = employee('2', 'Grace Hopper')

    expect(
      mentionPayloadsFromSelections(
        inferMentionSelections('@Ada Lovelace please pair with @Grace Hopper', [ada, grace], [
          { employeeId: '1', label: 'Ada Lovelace', start: 0, end: '@Ada Lovelace'.length },
        ]),
      ),
    ).toEqual([
      { employee_id: '1', label: 'Ada Lovelace' },
      { employee_id: '2', label: 'Grace Hopper' },
    ])
  })

  it('segments rendered comment text around persisted mentions', () => {
    expect(segmentCommentWithMentions('Pair with @Ada Lovelace on this.', [{ employee_id: '1', label: 'Ada Lovelace' }])).toEqual([
      { kind: 'text', value: 'Pair with ' },
      { kind: 'mention', value: '@Ada Lovelace', employeeId: '1' },
      { kind: 'text', value: ' on this.' },
    ])
  })

  it('linkifies mention tokens in markdown descriptions', () => {
    expect(linkifyMentionsInMarkdown('Check with @Ada Lovelace on this.', [{ employeeId: '1', label: 'Ada Lovelace' }])).toBe(
      'Check with [@Ada Lovelace](/employees/1) on this.',
    )
  })

  it('linkifies employee mentions using the employee directory', () => {
    expect(linkifyEmployeeMentionsInMarkdown('Need @Grace Hopper for review.', [employee('2', 'Grace Hopper')])).toBe(
      'Need [@Grace Hopper](/employees/2) for review.',
    )
  })

  it('linkifies issue identifiers, including inline-code references', () => {
    expect(linkifyIssueIdentifiersInMarkdown('See ISSUE-57 and `ISSUE-57`.', [{ issueId: 'issue-57', identifier: 'ISSUE-57' }])).toBe(
      'See [ISSUE-57](/issues/issue-57) and [`ISSUE-57`](/issues/issue-57).',
    )
  })

  it('linkifies raw employee mentions from the directory even without persisted mention payloads', () => {
    expect(linkifyEmployeeMentionsInMarkdown('Please sync with @CTO next.', [employee('cto-1', 'CTO')])).toBe(
      'Please sync with [@CTO](/employees/cto-1) next.',
    )
  })

  it('formats issue mention run triggers', () => {
    expect(formatRunTrigger({ issue_mention: { issue_id: 'issue-1', comment_id: 'comment-1' } })).toBe('Issue mention')
  })
})