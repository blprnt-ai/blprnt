import dayjs from 'dayjs'
import relativeTime from 'dayjs/plugin/relativeTime'
import type { RunStatus } from '@/bindings/RunStatus'
import type { RunTrigger } from '@/bindings/RunTrigger'

dayjs.extend(relativeTime)

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
  if (typeof trigger === 'object' && 'issue_assignment' in trigger) return 'Issue assignment'
  return 'Unknown'
}

export const formatRunTime = (date: Date | null) => {
  if (!date || Number.isNaN(date.getTime())) return 'Not started'
  return dayjs(date).fromNow()
}
