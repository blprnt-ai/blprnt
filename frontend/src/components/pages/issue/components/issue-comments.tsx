import { observer } from 'mobx-react-lite'
import { useAppViewmodel } from '@/app.viewmodel'
import { useIssueViewmodel } from '../issue.viewmodel'
import { EmptyState } from './empty-state'
import { IssueCommentCard } from './issue-comment-card'
import { IssueRunCard } from './issue-run-card'

export const IssueComments = observer(() => {
  const viewmodel = useIssueViewmodel()
  const appViewmodel = useAppViewmodel()

  if (viewmodel.timelineItems.length === 0) {
    return (
      <EmptyState
        description="Start the conversation by adding a comment, a decision, or a blocker."
        title="No comments yet"
      />
    )
  }

  return (
    <div className="space-y-3">
      {viewmodel.timelineItems.map((item) => {
        if (item.type === 'comment') {
          return (
            <IssueCommentCard
              key={`comment-${item.comment.id || item.comment.createdAt.toISOString()}`}
              comment={item.comment}
            />
          )
        }

        return (
          <IssueRunCard
            key={`run-${item.run.id}`}
            latestActivity={appViewmodel.runs.latestActivity(item.run.id)}
            run={item.run}
          />
        )
      })}
    </div>
  )
})
