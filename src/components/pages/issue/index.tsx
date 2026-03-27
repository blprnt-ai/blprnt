import { useParams } from '@tanstack/react-router'
import {
  ActivityIcon,
  ChevronRightIcon,
  FileIcon,
  GitBranchPlusIcon,
  MessageSquareIcon,
  PaperclipIcon,
} from 'lucide-react'
import { observer } from 'mobx-react-lite'
import type * as React from 'react'
import { useEffect, useRef, useState } from 'react'
import type { IssueActionKind } from '@/bindings/IssueActionKind'
import type { IssuePriority } from '@/bindings/IssuePriority'
import type { IssueStatus } from '@/bindings/IssueStatus'
import { Page } from '@/components/layouts/page'
import { AppLoader } from '@/components/organisms/app-loader'
import { MarkdownEditor, MarkdownEditorPreview } from '@/components/organisms/markdown-editor'
import { Avatar, AvatarFallback } from '@/components/ui/avatar'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Input } from '@/components/ui/input'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { Textarea } from '@/components/ui/textarea'
import { cn } from '@/lib/utils'
import { AppModel } from '@/models/app.model'
import { IssuePageViewmodel } from './issue.viewmodel'

export const IssuePage = observer(() => {
  const { issueId } = useParams({ from: '/issues/$issueId/' })
  const [viewmodel, setViewmodel] = useState<IssuePageViewmodel | null>(null)
  const [activeTab, setActiveTab] = useState('comments')
  const [isEditingTitle, setIsEditingTitle] = useState(false)
  const [isEditingDescription, setIsEditingDescription] = useState(false)
  const [titleDraft, setTitleDraft] = useState('')
  const [descriptionDraft, setDescriptionDraft] = useState('')
  const attachmentInputRef = useRef<HTMLInputElement>(null)

  useEffect(() => {
    const nextViewmodel = new IssuePageViewmodel(issueId)

    setViewmodel(nextViewmodel)
    void nextViewmodel.init()
  }, [issueId])

  useEffect(() => {
    if (!viewmodel?.issue) return

    setTitleDraft(viewmodel.issue.title)
    setDescriptionDraft(viewmodel.issue.description.replace(/\n/g, '\n\n'))
    setIsEditingTitle(false)
    setIsEditingDescription(false)
  }, [viewmodel?.issue])

  if (!viewmodel || viewmodel.isLoading) return <AppLoader />

  if (!viewmodel.issue) {
    return (
      <Page className="min-h-screen">
        <Card className="max-w-3xl">
          <CardHeader>
            <CardTitle>Issue unavailable</CardTitle>
            <CardDescription>{viewmodel.errorMessage ?? 'We could not load this issue.'}</CardDescription>
          </CardHeader>
        </Card>
      </Page>
    )
  }

  const { issue } = viewmodel
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
    { label: 'In review', value: 'in_review' },
    { label: 'Blocked', value: 'blocked' },
    { label: 'Done', value: 'done' },
    { label: 'Cancelled', value: 'cancelled' },
    { label: 'Archived', value: 'archived' },
  ]
  const resolveEmployeeName = (employeeId: string | null | undefined, fallback: string) => {
    return AppModel.instance.resolveEmployeeName(employeeId) ?? fallback
  }

  const handleAttachmentChange = async (event: React.ChangeEvent<HTMLInputElement>) => {
    const files = Array.from(event.target.files ?? [])
    if (files.length === 0) return

    await viewmodel.addAttachments(files)
    event.target.value = ''
  }

  const handleSaveTitle = async () => {
    const nextTitle = titleDraft.trim()
    if (nextTitle.length === 0) return

    const savedIssue = await viewmodel.saveTitle(nextTitle)
    if (!savedIssue) return

    setTitleDraft(savedIssue.title)
    setIsEditingTitle(false)
  }

  const handleCancelTitle = () => {
    setTitleDraft(issue.title)
    setIsEditingTitle(false)
  }

  const handleSaveDescription = async () => {
    const nextDescription = descriptionDraft.trim()
    if (nextDescription.length === 0) return

    const savedIssue = await viewmodel.saveDescription(descriptionDraft.replace(/\n\n/g, '\n'))
    if (!savedIssue) return

    setDescriptionDraft(savedIssue.description.replace(/\n/g, '\n\n'))
    setIsEditingDescription(false)
  }

  const handleCancelDescription = () => {
    setDescriptionDraft(issue.description.replace(/\n/g, '\n\n'))
    setIsEditingDescription(false)
  }

  return (
    <Page className="p-1 pr-2 overflow-y-auto">
      <div className="grid gap-3 xl:grid-cols-[minmax(0,1fr)_340px]">
        <div className="flex min-w-0 flex-col gap-3">
          <Card>
            <CardContent className="flex flex-col gap-6">
              <div className="flex flex-wrap items-center gap-2 text-xs uppercase tracking-[0.18em] text-muted-foreground">
                <span>{issue.identifier}</span>
                <IssueBadge>{formatLabel(issue.status)}</IssueBadge>
                <IssueBadge>{formatLabel(issue.priority)}</IssueBadge>
              </div>
              <div className="flex flex-col gap-4">
                {isEditingTitle ? (
                  <div className="space-y-3">
                    <Input
                      autoFocus
                      className="h-11 text-xl font-medium"
                      placeholder="Issue title"
                      value={titleDraft}
                      onChange={(event) => setTitleDraft(event.target.value)}
                    />
                    <div className="flex items-center justify-end gap-2">
                      <Button size="sm" variant="ghost" onClick={handleCancelTitle}>
                        Cancel
                      </Button>
                      <Button
                        disabled={titleDraft.trim().length === 0 || viewmodel.isSavingTitle}
                        size="sm"
                        onClick={() => void handleSaveTitle()}
                      >
                        {viewmodel.isSavingTitle ? 'Saving...' : 'Save'}
                      </Button>
                    </div>
                  </div>
                ) : (
                  <button
                    className="w-full rounded-md p-2 text-left transition-colors hover:bg-muted/60 focus-visible:bg-muted/30 focus-visible:outline-none duration-300"
                    type="button"
                    onClick={() => setIsEditingTitle(true)}
                  >
                    <CardTitle className="text-2xl">{issue.title || 'Untitled issue'}</CardTitle>
                  </button>
                )}

                {isEditingDescription ? (
                  <div className="flex flex-col gap-4">
                    <MarkdownEditor
                      className="min-h-[320px]"
                      placeholder="Describe the issue, context, and expected outcome..."
                      value={descriptionDraft}
                      onChange={setDescriptionDraft}
                    />
                    <div className="flex items-center justify-end gap-2">
                      <Button size="sm" variant="ghost" onClick={handleCancelDescription}>
                        Cancel
                      </Button>
                      <Button
                        disabled={descriptionDraft.trim().length === 0 || viewmodel.isSavingDescription}
                        size="sm"
                        onClick={() => void handleSaveDescription()}
                      >
                        {viewmodel.isSavingDescription ? 'Saving...' : 'Save'}
                      </Button>
                    </div>
                  </div>
                ) : (
                  <button
                    type="button"
                    className={cn(
                      'w-full rounded-md text-left transition-colors hover:bg-muted/60 focus-visible:bg-muted/30 focus-visible:outline-none duration-300',
                    )}
                    onClick={() => setIsEditingDescription(true)}
                  >
                    <MarkdownEditorPreview
                      value={issue.description.replace(/\n/g, '\n\n') || 'No description has been added yet.'}
                    />
                  </button>
                )}
              </div>
            </CardContent>
          </Card>

          <Card>
            <CardContent>
              <Tabs value={activeTab} onValueChange={setActiveTab}>
                <TabsList variant="line">
                  <TabsTrigger value="comments">
                    <MessageSquareIcon className="size-4" />
                    Comments
                  </TabsTrigger>
                  <TabsTrigger value="children">
                    <GitBranchPlusIcon className="size-4" />
                    Children
                  </TabsTrigger>
                  <TabsTrigger value="activity">
                    <ActivityIcon className="size-4" />
                    Activity
                  </TabsTrigger>
                </TabsList>

                <TabsContent className="mt-5 space-y-4" value="comments">
                  <div className="space-y-3">
                    {issue.comments.length > 0 ? (
                      issue.comments
                        .slice()
                        .reverse()
                        .map((comment) => (
                          <article
                            key={comment.id || comment.createdAt.toISOString()}
                            className="rounded-sm border border-border/60 p-4"
                          >
                            <div className="flex items-start gap-3">
                              <Avatar>
                                <AvatarFallback>
                                  {getInitials(resolveEmployeeName(comment.creator, 'You'))}
                                </AvatarFallback>
                              </Avatar>

                              <div className="min-w-0 flex-1 space-y-2">
                                <div className="flex flex-wrap items-center justify-between gap-2">
                                  <div className="font-medium">{resolveEmployeeName(comment.creator, 'You')}</div>
                                  <div className="text-xs text-muted-foreground">{formatDate(comment.createdAt)}</div>
                                </div>
                                <p className="whitespace-pre-wrap text-sm leading-6 text-foreground/90">
                                  {comment.comment}
                                </p>
                              </div>
                            </div>
                          </article>
                        ))
                    ) : (
                      <EmptyState
                        description="Start the conversation by adding a comment, a decision, or a blocker."
                        title="No comments yet"
                      />
                    )}
                  </div>

                  <div>
                    <form
                      onSubmit={(event) => {
                        event.preventDefault()
                        void viewmodel.submitComment()
                      }}
                    >
                      <Textarea
                        maxRows={8}
                        minRows={4}
                        placeholder="Add context, decisions, or next steps..."
                        value={viewmodel.commentDraft}
                        onChange={(event) => viewmodel.setCommentDraft(event.target.value)}
                      />

                      <div className="mt-4 flex flex-wrap items-center justify-between gap-3">
                        <div className="flex flex-wrap items-center gap-2">
                          <input
                            ref={attachmentInputRef}
                            multiple
                            className="hidden"
                            type="file"
                            onChange={(event) => {
                              void handleAttachmentChange(event)
                            }}
                          />
                          <Button type="button" variant="outline" onClick={() => attachmentInputRef.current?.click()}>
                            <PaperclipIcon className="size-4" />
                            {viewmodel.isUploadingAttachments ? 'Uploading...' : 'Add attachment'}
                          </Button>
                          <span className="text-xs text-muted-foreground">
                            Upload screenshots, specs, logs, or related files.
                          </span>
                        </div>

                        <Button disabled={!viewmodel.canSubmitComment} type="submit">
                          {viewmodel.isSubmittingComment ? 'Posting...' : 'Post comment'}
                        </Button>
                      </div>
                    </form>
                  </div>

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
                                <div className="truncate font-medium">
                                  {attachment.attachment.name || 'Untitled attachment'}
                                </div>
                                <div className="text-xs text-muted-foreground">
                                  {formatBytes(attachment.attachment.size)} ·{' '}
                                  {resolveEmployeeName(attachment.creator, 'You')} · {formatDate(attachment.createdAt)}
                                </div>
                              </div>
                            </div>
                            <span className="text-xs text-muted-foreground">Open</span>
                          </a>
                        ))
                    ) : (
                      <EmptyState
                        description="Uploaded files will appear here for quick reference."
                        title="No attachments yet"
                      />
                    )}
                  </div>
                </TabsContent>

                <TabsContent className="mt-5" value="children">
                  <div className="space-y-3">
                    <div className="flex items-center justify-between gap-3">
                      <div>
                        <h3 className="text-sm font-medium">Child issues</h3>
                        <p className="text-sm text-muted-foreground">
                          Track smaller pieces of work linked to this issue.
                        </p>
                      </div>
                      <IssueBadge>{viewmodel.childIssues.length} total</IssueBadge>
                    </div>

                    {viewmodel.isLoadingChildIssues ? (
                      <EmptyState description="Loading linked child issues..." title="Fetching child issues" />
                    ) : viewmodel.childIssues.length > 0 ? (
                      viewmodel.childIssues.map((childIssue) => (
                        <div
                          key={childIssue.id || childIssue.identifier}
                          className="rounded-sm border border-border/60 p-4 transition-colors hover:bg-muted/20"
                        >
                          <div className="flex items-start justify-between gap-3">
                            <div className="min-w-0 flex-1">
                              <div className="flex flex-wrap items-center gap-2 text-xs uppercase tracking-[0.18em] text-muted-foreground">
                                <span>{childIssue.identifier || childIssue.id || 'Child issue'}</span>
                                <IssueBadge>{formatLabel(childIssue.status)}</IssueBadge>
                                <IssueBadge>{formatLabel(childIssue.priority)}</IssueBadge>
                              </div>
                              <div className="mt-2 text-sm font-medium">
                                {childIssue.title || 'Untitled child issue'}
                              </div>
                              <p className="mt-1 line-clamp-2 text-sm text-muted-foreground">
                                {childIssue.description || 'No description yet.'}
                              </p>
                            </div>
                            <div className="flex items-center gap-1 text-xs text-muted-foreground">
                              <span>{resolveEmployeeName(childIssue.assignee, 'Unassigned')}</span>
                              <ChevronRightIcon className="size-4" />
                            </div>
                          </div>
                        </div>
                      ))
                    ) : (
                      <EmptyState
                        description="Child issues will appear here once this issue is broken into smaller tasks."
                        title="No child issues yet"
                      />
                    )}
                  </div>
                </TabsContent>

                <TabsContent className="mt-5" value="activity">
                  <div className="space-y-3">
                    {issue.actions.length > 0 ? (
                      issue.actions
                        .slice()
                        .reverse()
                        .map((action) => (
                          <article
                            key={action.id || action.createdAt.toISOString()}
                            className="flex gap-3 rounded-sm border border-border/60 p-4"
                          >
                            <div className="mt-0.5 flex size-9 shrink-0 items-center justify-center rounded-full bg-muted text-muted-foreground">
                              <ActivityIcon className="size-4" />
                            </div>
                            <div className="min-w-0 flex-1">
                              <p className="text-sm font-medium">{formatAction(action.action)}</p>
                              <p className="mt-1 text-sm text-muted-foreground">
                                {resolveEmployeeName(action.creator, 'System')} · {formatDate(action.createdAt)}
                              </p>
                            </div>
                          </article>
                        ))
                    ) : (
                      <EmptyState
                        description="Actions like status updates, assignments, and uploads will appear here."
                        title="No activity yet"
                      />
                    )}
                  </div>
                </TabsContent>
              </Tabs>
            </CardContent>
          </Card>
        </div>

        <Card className="h-fit">
          <CardContent className="space-y-4">
            <EditableMetadataSelect
              label="Project"
              options={projectOptions}
              placeholder="Select a project"
              value={issue.project}
              onValueChange={(value) => {
                issue.project = value
                void viewmodel.saveMetadata()
              }}
            />
            <EditableMetadataSelect
              label="Assignee"
              options={assigneeOptions}
              placeholder="Select an assignee"
              value={issue.assignee}
              onValueChange={(value) => {
                issue.assignee = value
                void viewmodel.saveMetadata()
              }}
            />
            <EditableMetadataSelect
              label="Priority"
              options={priorityOptions}
              value={issue.priority}
              onValueChange={(value) => {
                issue.priority = value as IssuePriority
                void viewmodel.saveMetadata()
              }}
            />
            <EditableMetadataSelect
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
      </div>
    </Page>
  )
})

const MetadataRow = ({ label, value }: { label: string; value: string }) => {
  return (
    <div className="flex items-start gap-3">
      <div className="min-w-0">
        <div className="text-xs uppercase tracking-[0.18em] text-muted-foreground/50">{label}</div>
        <div className="mt-1 wrap-break-word text-sm font-medium text-muted-foreground/90">{value}</div>
      </div>
    </div>
  )
}

const EditableMetadataSelect = ({
  label,
  onValueChange,
  options,
  placeholder,
  value,
}: {
  label: string
  onValueChange: (value: string) => void
  options: { label: string; value: string }[]
  placeholder?: string
  value: string
}) => {
  const selectedLabel = options.find((option) => option.value === value)?.label

  return (
    <div className="flex items-start">
      <div className="min-w-0 flex-1">
        <div className="text-xs uppercase tracking-[0.18em] text-muted-foreground/50">{label}</div>
        <Select
          value={value}
          onValueChange={(nextValue) => {
            onValueChange(nextValue ?? '')
          }}
        >
          <SelectTrigger className="w-full border-none text-muted-foreground/90 bg-transparent! pl-0" size="sm">
            <SelectValue placeholder={placeholder}>{selectedLabel}</SelectValue>
          </SelectTrigger>
          <SelectContent>
            {options.map((option) => (
              <SelectItem key={option.value} value={option.value}>
                {option.label}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
      </div>
    </div>
  )
}

const EmptyState = ({ title, description }: { title: string; description: string }) => {
  return (
    <div className="rounded-sm border border-dashed border-border/70 p-6 text-center">
      <div className="font-medium">{title}</div>
      <p className="mt-1 text-sm text-muted-foreground">{description}</p>
    </div>
  )
}

const IssueBadge = ({ children }: { children: React.ReactNode }) => {
  return (
    <span className="rounded-full border border-border/60 bg-muted/30 px-2.5 py-1 text-[11px] font-medium">
      {children}
    </span>
  )
}

const formatLabel = (value: string) => {
  return value
    .split(/[_-]/g)
    .filter(Boolean)
    .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
    .join(' ')
}

const formatDate = (value: Date) => {
  if (Number.isNaN(value.getTime())) return 'Unknown'

  return new Intl.DateTimeFormat('en', {
    dateStyle: 'medium',
    timeStyle: 'short',
  }).format(value)
}

const formatAction = (action: IssueActionKind) => {
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

const formatBytes = (value: number) => {
  if (value < 1024) return `${value} B`
  if (value < 1024 * 1024) return `${Math.round(value / 102.4) / 10} KB`

  return `${Math.round(value / (1024 * 102.4)) / 10} MB`
}

const getInitials = (value: string) => {
  const parts = value.trim().split(/\s+/).filter(Boolean)
  if (parts.length === 0) return 'U'

  return parts
    .slice(0, 2)
    .map((part) => part.charAt(0).toUpperCase())
    .join('')
}
