import { FileIcon } from 'lucide-react'
import { useIssueViewmodel } from '../issue.viewmodel'
import { formatBytes, formatDate, resolveEmployeeName } from '../utils'
import { EmptyState } from './empty-state'
import { IssueBadge } from './issue-badge'

export const IssueAttachments = () => {
  const viewmodel = useIssueViewmodel()

  const { issue } = viewmodel
  if (!issue) return null

  return (
    <div className="space-y-3">
      <div className="flex items-center justify-between gap-3">
        <div>
          <h3 className="text-sm font-medium">Attachments</h3>
          <p className="text-sm text-muted-foreground">Files uploaded to support the issue.</p>
        </div>
        <IssueBadge>{issue.attachments.length} total</IssueBadge>
      </div>

      {issue.attachments.length > 0 ? (
        issue.attachments
          .slice()
          .reverse()
          .map((attachment) => (
            <a
              key={attachment.id || attachment.createdAt.toISOString()}
              className="flex items-center justify-between gap-3 rounded-sm border border-border/60 p-4 transition-colors hover:bg-muted/30"
              href={attachment.attachment.attachment}
              rel="noreferrer"
              target="_blank"
            >
              <div className="flex min-w-0 items-center gap-3">
                <div className="flex size-10 items-center justify-center rounded-md bg-muted text-muted-foreground">
                  <FileIcon className="size-4" />
                </div>
                <div className="min-w-0">
                  <div className="truncate font-medium">{attachment.attachment.name || 'Untitled attachment'}</div>
                  <div className="text-xs text-muted-foreground">
                    {formatBytes(attachment.attachment.size)} · {resolveEmployeeName(attachment.creator, 'You')} ·{' '}
                    {formatDate(attachment.createdAt)}
                  </div>
                </div>
              </div>
              <span className="text-xs text-muted-foreground">Open</span>
            </a>
          ))
      ) : (
        <EmptyState description="Uploaded files will appear here for quick reference." title="No attachments yet" />
      )}
    </div>
  )
}
