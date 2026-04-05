import type { IssueActionKind } from '@/bindings/IssueActionKind'
import { AppModel } from '@/models/app.model'

export const formatLabel = (value: string) => {
  return value
    .split(/[_-]/g)
    .filter(Boolean)
    .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
    .join(' ')
}

export const formatDate = (value: Date) => {
  if (Number.isNaN(value.getTime())) return 'Unknown'

  return new Intl.DateTimeFormat('en', {
    dateStyle: 'medium',
    timeStyle: 'short',
  }).format(value)
}

export const formatAction = (action: IssueActionKind) => {
  if (typeof action === 'string') {
    return formatLabel(action)
  }

  if ('assign' in action) {
    return `Assigned to ${action.assign.employee}`
  }

  if ('status_change' in action) {
    return `Status changed from ${formatLabel(action.status_change.from)} to ${formatLabel(action.status_change.to)}`
  }

  return 'Updated issue'
}

export const formatBytes = (value: number) => {
  if (value < 1024) return `${value} B`
  if (value < 1024 * 1024) return `${Math.round(value / 102.4) / 10} KB`

  return `${Math.round(value / (1024 * 102.4)) / 10} MB`
}

export const getInitials = (value: string) => {
  const parts = value.trim().split(/\s+/).filter(Boolean)
  if (parts.length === 0) return 'U'

  return parts
    .slice(0, 2)
    .map((part) => part.charAt(0).toUpperCase())
    .join('')
}

export const resolveEmployeeName = (employeeId: string | null | undefined, fallback: string) => {
  return AppModel.instance.resolveEmployeeName(employeeId) ?? fallback
}
