import { MarkdownEditorPreview } from '@/components/organisms/markdown-editor'
import { AppModel } from '@/models/app.model'
import type { IssueCommentModel } from '@/models/issue-comment.model'
import { linkifyEmployeeMentionsInMarkdown, linkifyIssueIdentifiersInMarkdown, linkifyMentionsInMarkdown } from '../comment-mentions'

interface IssueCommentBodyProps {
  comment: IssueCommentModel
}

export const IssueCommentBody = ({ comment }: IssueCommentBodyProps) => {
  const linkedComment = linkifyIssueIdentifiersInMarkdown(
    linkifyEmployeeMentionsInMarkdown(
      linkifyMentionsInMarkdown(
        comment.comment,
        comment.mentions.map((mention) => ({ employeeId: mention.employee_id, label: mention.label })),
      ),
      AppModel.instance.employees,
    ),
    AppModel.instance.issues.map((knownIssue) => ({ issueId: knownIssue.id, identifier: knownIssue.identifier })),
  )

  return <MarkdownEditorPreview value={linkedComment} />
}
