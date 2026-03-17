import { MarkdownEditor } from '@/components/organisms/markdown-editor'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import type { IssueFormViewmodel } from './issue-form.viewmodel'

interface IssueFormFieldsProps {
  viewmodel: IssueFormViewmodel
}

export const IssueFormFields = ({ viewmodel }: IssueFormFieldsProps) => {
  return (
    <div className="flex flex-1 flex-col gap-6 overflow-y-auto px-4 pb-4">
      <div className="flex flex-col gap-2">
        <Label htmlFor="issue-create-title">Title</Label>
        <Input
          autoFocus
          id="issue-create-title"
          placeholder="Kick off roadmap planning"
          value={viewmodel.issue.title}
          onChange={(event) => {
            viewmodel.issue.title = event.target.value
          }}
        />
      </div>

      <div className="flex flex-col gap-2">
        <Label>Description</Label>
        <MarkdownEditor
          placeholder="Describe the issue, context, and expected outcome..."
          value={viewmodel.issue.description}
          onChange={(value) => {
            viewmodel.issue.description = value
          }}
        />
      </div>

      <div className="grid gap-4 md:grid-cols-2">
        <div className="flex flex-col gap-2">
          <Label>Project</Label>
          <Select
            value={viewmodel.issue.project}
            onValueChange={(value) => {
              viewmodel.issue.project = value ?? ''
            }}
          >
            <SelectTrigger className="w-full">
              <SelectValue placeholder="No project">
                {viewmodel.projectOptions.find((option) => option.value === viewmodel.issue.project)?.label}
              </SelectValue>
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="">No project</SelectItem>
              {viewmodel.projectOptions.map((option) => (
                <SelectItem key={option.value} value={option.value}>
                  {option.label}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>

        <div className="flex flex-col gap-2">
          <Label>Assignee</Label>
          <Select
            value={viewmodel.issue.assignee}
            onValueChange={(value) => {
              viewmodel.issue.assignee = value ?? ''
            }}
          >
            <SelectTrigger className="w-full">
              <SelectValue placeholder="Unassigned">
                {viewmodel.assigneeOptions.find((option) => option.value === viewmodel.issue.assignee)?.label}
              </SelectValue>
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="">Unassigned</SelectItem>
              {viewmodel.assigneeOptions.map((option) => (
                <SelectItem key={option.value} value={option.value}>
                  {option.label}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>

        <div className="flex flex-col gap-2">
          <Label>Priority</Label>
          <Select
            value={viewmodel.issue.priority}
            onValueChange={(value) => {
              viewmodel.issue.priority = (value ?? 'medium') as typeof viewmodel.issue.priority
            }}
          >
            <SelectTrigger className="w-full">
              <SelectValue>
                {viewmodel.priorityOptions.find((option) => option.value === viewmodel.issue.priority)?.label}
              </SelectValue>
            </SelectTrigger>
            <SelectContent>
              {viewmodel.priorityOptions.map((option) => (
                <SelectItem key={option.value} value={option.value}>
                  {option.label}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>
      </div>
    </div>
  )
}
