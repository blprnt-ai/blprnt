import { AlertTriangle, CheckCircle2, LoaderCircle, Upload, X } from 'lucide-react'
import { observer } from 'mobx-react-lite'
import type { ChangeEvent, ClipboardEvent, FormEvent, ReactNode } from 'react'
import { useEffect, useId, useRef } from 'react'
import { Button } from '@/components/atoms/button'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/atoms/dialog'
import { Input } from '@/components/atoms/input'
import { Label } from '@/components/atoms/label'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/atoms/select'
import { Textarea } from '@/components/atoms/textarea'
import { cn } from '@/lib/utils/cn'
import type { ReportBugDialogViewModel } from './report-bug-dialog.viewmodel'

interface ReportBugDialogProps {
  viewmodel: ReportBugDialogViewModel
}

const severityOptions = [
  { label: 'Low', value: 'LOW' },
  { label: 'Medium', value: 'MEDIUM' },
  { label: 'High', value: 'HIGH' },
  { label: 'Critical', value: 'CRITICAL' },
] as const

export const ReportBugDialog = observer(({ viewmodel }: ReportBugDialogProps) => {
  const screenshotInputId = useId()
  const screenshotInputRef = useRef<HTMLInputElement>(null)

  useEffect(() => {
    void viewmodel.init()
    return () => viewmodel.destroy()
  }, [viewmodel])

  const handleSubmit = async (event: FormEvent) => {
    event.preventDefault()
    await viewmodel.submit()
  }

  const handleScreenshotPicker = () => {
    screenshotInputRef.current?.click()
  }

  const handleScreenshotChange = (event: ChangeEvent<HTMLInputElement>) => {
    const file = event.currentTarget.files?.[0] ?? null
    viewmodel.setScreenshotFile(file)
    event.currentTarget.value = ''
  }

  const handleDialogClose = (isOpen: boolean) => {
    viewmodel.onOpenChange(isOpen)
  }

  const handleDescriptionPaste = (event: ClipboardEvent<HTMLTextAreaElement>) => {
    const items = event.clipboardData?.items
    if (!items?.length) return

    const files: File[] = []
    for (const item of items) {
      const file = item.getAsFile()
      if (!file) continue
      files.push(file)
    }

    if (files.length === 0) return
    viewmodel.queuePastedFiles(files)
  }

  return (
    <Dialog open={viewmodel.isOpen} onOpenChange={handleDialogClose}>
      <DialogContent size="lg">
        <DialogHeader>
          <DialogTitle>Report Bug</DialogTitle>
          <DialogDescription>Submit a structured bug report with required details.</DialogDescription>
        </DialogHeader>

        <form className="space-y-4" onSubmit={handleSubmit}>
          <StateBanner viewmodel={viewmodel} />

          <div className="space-y-2">
            <Label htmlFor="report-bug-title">Title</Label>
            <Input
              aria-invalid={Boolean(viewmodel.validationErrors.title)}
              disabled={viewmodel.isSubmitting}
              id="report-bug-title"
              placeholder="Short summary"
              value={viewmodel.title}
              onChange={(event) => viewmodel.setTitle(event.target.value)}
            />
            <ValidationMessage message={viewmodel.validationErrors.title} />
          </div>

          <div className="space-y-2">
            <Label htmlFor="report-bug-description">Description</Label>
            <Textarea
              aria-invalid={Boolean(viewmodel.validationErrors.description)}
              className="min-h-28"
              disabled={viewmodel.isSubmitting}
              placeholder="What happened, expected behavior, reproduction steps"
              value={viewmodel.description}
              onChange={(event) => viewmodel.setDescription(event.target.value)}
              onPaste={handleDescriptionPaste}
            />
            <ValidationMessage message={viewmodel.validationErrors.description} />

            {viewmodel.queuedPastedAttachments.length > 0 && (
              <div className="space-y-2 rounded-md border border-dashed p-2">
                <div className="text-xs font-medium">Queued pasted attachments</div>
                <div className="space-y-1">
                  {viewmodel.queuedPastedAttachments.map((attachment) => (
                    <div key={attachment.clientId} className="flex items-center justify-between gap-2 text-xs">
                      <span className="truncate">
                        {attachment.fileName} ({Math.ceil(attachment.byteLen / 1024)} KB)
                      </span>
                      <Button
                        disabled={viewmodel.isSubmitting}
                        size="sm"
                        type="button"
                        variant="ghost"
                        onClick={() => viewmodel.removeQueuedPastedAttachment(attachment.clientId)}
                      >
                        <X className="size-3.5" />
                        Remove
                      </Button>
                    </div>
                  ))}
                </div>
              </div>
            )}

            {viewmodel.rejectedPastedAttachments.length > 0 && (
              <div className="space-y-1 rounded-md border border-destructive/50 p-2 text-xs text-destructive">
                {viewmodel.rejectedPastedAttachments.map((attachment) => (
                  <div key={attachment.clientId} className="flex items-center justify-between gap-2">
                    <span className="truncate">
                      {attachment.fileName}: {attachment.message}
                    </span>
                    <Button
                      disabled={viewmodel.isSubmitting}
                      size="sm"
                      type="button"
                      variant="ghost"
                      onClick={() => viewmodel.dismissRejectedPastedAttachment(attachment.clientId)}
                    >
                      <X className="size-3.5" />
                    </Button>
                  </div>
                ))}
              </div>
            )}
          </div>

          <div className="space-y-2">
            <Label>Severity</Label>
            <Select
              disabled={viewmodel.isSubmitting}
              value={viewmodel.severity}
              onValueChange={viewmodel.setSeverity}
            >
              <SelectTrigger aria-invalid={Boolean(viewmodel.validationErrors.severity)} className="w-full">
                <SelectValue placeholder="Select severity" />
              </SelectTrigger>
              <SelectContent>
                {severityOptions.map((severityOption) => (
                  <SelectItem key={severityOption.value} value={severityOption.value}>
                    {severityOption.label}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
            <ValidationMessage message={viewmodel.validationErrors.severity} />
          </div>

          <div className="space-y-2">
            <Label htmlFor={screenshotInputId}>Screenshot (optional)</Label>
            <input
              ref={screenshotInputRef}
              accept="image/*"
              className="hidden"
              id={screenshotInputId}
              type="file"
              onChange={handleScreenshotChange}
            />
            {!viewmodel.screenshotFile && (
              <Button
                className="w-full"
                disabled={viewmodel.isSubmitting}
                type="button"
                variant="outline"
                onClick={handleScreenshotPicker}
              >
                <Upload className="size-4" />
                Pick Screenshot
              </Button>
            )}

            {viewmodel.screenshotFile && (
              <div className="rounded-md border border-dashed p-2">
                {viewmodel.screenshotPreviewUrl && (
                  <img
                    alt="Selected bug screenshot"
                    className="max-h-52 w-full rounded-md border object-contain"
                    src={viewmodel.screenshotPreviewUrl}
                  />
                )}
                <div className="mt-2 flex items-center justify-between text-xs text-muted-foreground">
                  <span>{viewmodel.screenshotFile.name}</span>
                  <Button
                    disabled={viewmodel.isSubmitting}
                    size="sm"
                    type="button"
                    variant="ghost"
                    onClick={viewmodel.removeScreenshot}
                  >
                    <X className="size-3.5" />
                    Remove
                  </Button>
                </div>
              </div>
            )}
          </div>

          <DialogFooter>
            <Button type="button" variant="ghost" onClick={() => viewmodel.close()}>
              Cancel
            </Button>
            {viewmodel.canRetry && (
              <Button disabled={viewmodel.isSubmitting} type="button" variant="secondary" onClick={viewmodel.retry}>
                Retry
              </Button>
            )}
            <Button disabled={!viewmodel.canSubmit} type="submit" variant="outline">
              {viewmodel.isSubmitting ? (
                <>
                  <LoaderCircle className="size-4 animate-spin" />
                  Submitting...
                </>
              ) : (
                'Submit'
              )}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
})

const ValidationMessage = ({ message }: { message: string | null }) => {
  if (!message) return null
  return <div className="text-xs text-destructive">{message}</div>
}

const StateBanner = observer(({ viewmodel }: ReportBugDialogProps) => {
  if (viewmodel.isSuccess) {
    return <Banner icon={<CheckCircle2 className="size-4" />} text="Bug report submitted." tone="success" />
  }

  if (viewmodel.isError) {
    return (
      <Banner
        icon={<AlertTriangle className="size-4" />}
        text={viewmodel.errorMessage ?? 'Bug report submission failed.'}
        tone="error"
      />
    )
  }

  if (viewmodel.shouldShowInvalidBanner) {
    return (
      <Banner
        icon={<AlertTriangle className="size-4" />}
        text="Fill all required fields before submitting."
        tone="warn"
      />
    )
  }

  return null
})

const Banner = ({
  icon,
  text,
  tone,
}: {
  icon: ReactNode
  text: string
  tone: 'error' | 'info' | 'success' | 'warn'
}) => {
  return (
    <div
      className={cn(
        'flex items-center gap-2 rounded-md border px-3 py-2 text-xs',
        tone === 'info' && 'border-primary/40 text-primary',
        tone === 'warn' && 'border-yellow-500/40 text-yellow-500',
        tone === 'success' && 'border-green-500/40 text-green-500',
        tone === 'error' && 'border-destructive/60 text-destructive',
      )}
    >
      {icon}
      <span>{text}</span>
    </div>
  )
}
