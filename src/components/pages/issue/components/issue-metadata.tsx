import type { IssuePriority } from '@/bindings/IssuePriority'
import type { IssueStatus } from '@/bindings/IssueStatus'
import { Card, CardContent } from '@/components/ui/card'
import { AppModel } from '@/models/app.model'
import { useIssueViewmodel } from '../issue.viewmodel'
import { formatDate, resolveEmployeeName } from '../utils'
import { EditableMetadata } from './editable-metadata'
import { MetadataRow } from './metadata-row'

export const IssueMetadata = () => {
  const viewmodel = useIssueViewmodel()

  const { issue } = viewmodel

  if (!issue) return null

  const projectOptions = [
    { label: 'No project', value: '' },
    ...AppModel.instance.projects.map((project) => ({ label: project.name, value: project.id })),
  ]

  const assigneeOptions = [
    { label: 'Unassigned', value: '' },
    ...AppModel.instance.employees.map((employee) => ({ label: employee.name, value: employee.id })),
  ]

  const priorityOptions: { label: string; value: IssuePriority }[] = [
    { label: 'Low', value: 'low' },
    { label: 'Medium', value: 'medium' },
    { label: 'High', value: 'high' },
    { label: 'Critical', value: 'critical' },
  ]

  const statusOptions: { label: string; value: IssueStatus }[] = [
    { label: 'Backlog', value: 'backlog' },
    { label: 'Todo', value: 'todo' },
    { label: 'In progress', value: 'in_progress' },
    { label: 'Blocked', value: 'blocked' },
    { label: 'Done', value: 'done' },
    { label: 'Cancelled', value: 'cancelled' },
    { label: 'Archived', value: 'archived' },
  ]

  return (
    <Card className="h-fit">
      <CardContent className="space-y-4">
        <EditableMetadata
          label="Project"
          options={projectOptions}
          placeholder="Select a project"
          value={issue.project}
          onValueChange={(value) => {
            issue.project = value
            void viewmodel.saveMetadata()
          }}
        />
        <EditableMetadata
          label="Assignee"
          options={assigneeOptions}
          placeholder="Select an assignee"
          value={issue.assignee}
          onValueChange={(value) => {
            issue.assignee = value
            void viewmodel.saveMetadata()
          }}
        />
        <EditableMetadata
          label="Priority"
          options={priorityOptions}
          value={issue.priority}
          onValueChange={(value) => {
            issue.priority = value as IssuePriority
            void viewmodel.saveMetadata()
          }}
        />
        <EditableMetadata
          label="Status"
          options={statusOptions}
          value={issue.status}
          onValueChange={(value) => {
            issue.status = value as IssueStatus
            void viewmodel.saveMetadata()
          }}
        />

        <MetadataRow label="Creator" value={resolveEmployeeName(issue.creator, 'Unknown')} />

        {issue.checkedOutBy ? (
          <MetadataRow label="Checked out by" value={resolveEmployeeName(issue.checkedOutBy, 'Nobody')} />
        ) : null}

        {issue.blockedBy ? <MetadataRow label="Blocked by" value={issue.blockedBy} /> : null}
        {issue.parent ? <MetadataRow label="Parent issue" value={issue.parent} /> : null}
        <MetadataRow label="Created" value={formatDate(issue.createdAt)} />

        <MetadataRow label="Last updated" value={formatDate(issue.updatedAt)} />
      </CardContent>
    </Card>
  )
}
