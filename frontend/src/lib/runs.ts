import type { RunStatus } from '@/bindings/RunStatus'
import type { RunTrigger } from '@/bindings/RunTrigger'
import { formatRelativeTime } from './time'

export const formatRunStatus = (status: RunStatus) => {
  if (typeof status === 'string') return status.replace('_', ' ')
  return status.Failed ? 'failed' : 'unknown'
}

export const getRunFailureMessage = (status: RunStatus) => {
  if (typeof status === 'object' && 'Failed' in status) return status.Failed
  return null
}

export const runStatusTone = (status: RunStatus) => {
  if (status === 'Running') return 'text-emerald-600'
  if (status === 'Completed') return 'text-sky-600'
  if (status === 'Cancelled') return 'text-amber-600'
  if (typeof status === 'object' && 'Failed' in status) return 'text-destructive'
  return 'text-muted-foreground'
}

export const formatRunTrigger = (trigger: RunTrigger) => {
  if (trigger === 'manual') return 'Manual'
  if (trigger === 'conversation') return 'Conversation'
  if (trigger === 'timer') return 'Timer'
  if (trigger === 'dreaming') return 'Dreaming'
  if (typeof trigger === 'object' && 'issue_assignment' in trigger) return 'Issue assignment'
  if (typeof trigger === 'object' && 'issue_mention' in trigger) return 'Issue mention'
  return 'Unknown'
}

export interface RunIssueTarget {
  commentHash?: string
  issueId: string
}

export const getRunIssueTarget = (trigger: RunTrigger): RunIssueTarget | null => {
  if (typeof trigger !== 'object' || !trigger) return null

  if ('issue_assignment' in trigger) {
    const issueId = extractRecordUuid(trigger.issue_assignment.issue_id)
    return issueId ? { issueId } : null
  }

  if ('issue_mention' in trigger) {
    const issueId = extractRecordUuid(trigger.issue_mention.issue_id)
    if (!issueId) return null

    const commentId = extractRecordUuid(trigger.issue_mention.comment_id)
    return {
      commentHash: commentId ? `comment-${commentId}` : undefined,
      issueId,
    }
  }

  return null
}

export const formatRunTime = (date: Date | null) => {
  if (!date || Number.isNaN(date.getTime())) return 'Not started'
  return formatRelativeTime(date)
}

const extractRecordUuid = (value: string) => value.match(/[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}/i)?.[0] ?? null
