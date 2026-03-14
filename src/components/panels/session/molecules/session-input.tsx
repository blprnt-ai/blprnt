import { AnimatePresence, motion } from 'framer-motion'
import { LoaderCircleIcon, Pause, XIcon } from 'lucide-react'
import { Button } from '@/components/atoms/button'
import { Textarea } from '@/components/atoms/textarea'
import { stopSessionToast as toast } from '@/components/atoms/toaster'
import { TooltipMacro } from '@/components/atoms/tooltip-macro'
import { useSessionPanelViewmodel } from '@/components/panels/session/session-panel.viewmodel'
import { cn } from '@/lib/utils/cn'

export const SessionInput = () => {
  const viewmodel = useSessionPanelViewmodel()
  const session = viewmodel.session
  const queuedPrompts = viewmodel.queuedPrompts

  if (!session) return null

  const handleSubmit = async (event: React.FormEvent) => {
    event.preventDefault()
    await viewmodel.submitPrompt()
  }

  const handleStop = async () => {
    toast.loading({ title: 'Stopping session...' })
    await viewmodel.interrupt()
    toast.success({ title: 'Session stopped' })
  }

  const handlePaste = async (e: React.ClipboardEvent<HTMLTextAreaElement>) => {
    if (!e.clipboardData?.items?.length) return

    for (const item of e.clipboardData?.items ?? []) {
      if (item.type.startsWith('image/')) {
        const blob = item.getAsFile()
        if (!blob) continue

        const imgUrl = URL.createObjectURL(blob)
        const base64 = await toBase64(blob)

        viewmodel.addImageUrl(imgUrl, base64)
      }
    }
  }

  return (
    <AnimatePresence mode="wait">
      <motion.div
        animate={{ opacity: 1, x: 0, y: 0 }}
        className="relative mb-2 px-2 border-t pt-2"
        initial={{ opacity: 0, y: 10 }}
        transition={{ delay: 0.2, duration: 0.25 }}
      >
        <form data-tour="session-input" onSubmit={handleSubmit}>
          {viewmodel.imageUrls.size > 0 && (
            <div className="flex flex-wrap gap-2 mb-2">
              {Array.from(viewmodel.imageUrls.keys()).map((url) => (
                <div key={url} className="relative">
                  <img alt="Image" className="size-40 rounded-md border object-cover" src={url} />
                  <Button
                    className="absolute top-0 right-0"
                    size="icon"
                    variant="link"
                    onClick={() => viewmodel.removeImageUrl(url)}
                  >
                    <XIcon className="size-4" />
                  </Button>
                </div>
              ))}
            </div>
          )}

          {queuedPrompts.length > 0 && (
            <div className="rounded-md border-b-0 rounded-b-none border dark:border-success/40 bg-green-200/20 dark:bg-accent p-2 shadow-sm mx-4">
              <div className="flex max-h-44 flex-col gap-1.5 overflow-y-auto">
                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-2">
                    <span className="text-[10px] font-medium text-muted-foreground/70">Queue</span>
                  </div>
                </div>

                {queuedPrompts.map((message) => (
                  <div key={message.id} className={cn(message.isDeleting && 'opacity-70')}>
                    {message.imageUrls.length > 0 && (
                      <div className="mb-1 flex flex-wrap gap-1.5 border-b border-dashed border-border pb-1.5">
                        {message.imageUrls.map((url) => (
                          <img
                            key={url}
                            alt="Image"
                            className="size-10 rounded-md border border-border object-cover"
                            src={url}
                          />
                        ))}
                      </div>
                    )}

                    <div className="flex justify-between items-start gap-2">
                      <div className="line-clamp-3 whitespace-pre-wrap text-sm text-muted-foreground/80 animate-pulse">
                        {message.content}
                      </div>
                      <Button
                        className="size-6 shrink-0"
                        disabled={message.isDeleting}
                        size="icon"
                        type="button"
                        variant="ghost"
                        onClick={() => void viewmodel.deleteQueuedPrompt(message.id)}
                      >
                        {message.isDeleting ? (
                          <LoaderCircleIcon className="size-3.5 animate-spin" />
                        ) : (
                          <XIcon className="size-3.5" />
                        )}
                      </Button>
                    </div>
                  </div>
                ))}
              </div>
            </div>
          )}

          <div className="relative border border-green-500 dark:border-success/80 bg-green-200/20 dark:bg-accent rounded-md p-1">
            {viewmodel.isSlashOpen && (
              <div className="absolute bottom-full left-0 right-0 mb-1 rounded-md border border-border bg-card shadow-lg">
                <div className="px-3 py-2 text-[11px] text-muted-foreground/80 border-b border-border">
                  /{viewmodel.slashPickerQuery || '...'}
                </div>
                <div className="max-h-48 overflow-y-auto py-1">
                  {viewmodel.isSlashLoading ? (
                    <div className="px-3 py-2 text-xs text-muted-foreground">Loading commands...</div>
                  ) : viewmodel.filteredSlashCommands.length === 0 ? (
                    <div className="px-3 py-2 text-xs text-muted-foreground">No matches</div>
                  ) : (
                    viewmodel.filteredSlashCommands.map((command, index) => (
                      <div
                        key={command.name}
                        className={cn(
                          'px-3 py-2 text-xs flex flex-col gap-0.5 cursor-default',
                          index === viewmodel.slashHighlight ? 'bg-accent text-foreground' : 'text-muted-foreground/80',
                        )}
                      >
                        <span className="font-medium text-foreground">/{command.name}</span>
                        <span className="text-[11px] text-muted-foreground/80">{command.description}</span>
                      </div>
                    ))
                  )}
                </div>
              </div>
            )}
            <Textarea
              className={cn('resize-none border-none max-h-64')}
              placeholder="Enter a prompt..."
              value={viewmodel.prompt}
              variant="no-focus"
              onChange={(e) => viewmodel.setPrompt(e.target.value)}
              onPaste={handlePaste}
              onKeyDown={(e) => {
                // if (viewmodel.isSlashOpen) {
                //   if (e.key === 'Escape') {
                //     e.preventDefault()
                //     viewmodel.closeSlashPicker()
                //     return
                //   }
                //   if (e.key === 'ArrowDown') {
                //     e.preventDefault()
                //     viewmodel.moveSlashHighlight(1)
                //     return
                //   }
                //   if (e.key === 'ArrowUp') {
                //     e.preventDefault()
                //     viewmodel.moveSlashHighlight(-1)
                //     return
                //   }
                //   if (e.key === 'Enter') {
                //     e.preventDefault()
                //     viewmodel.runSlashHighlighted()
                //     return
                //   }
                //   if (e.key === 'Tab') {
                //     e.preventDefault()
                //     viewmodel.autocompleteSlashHighlighted()
                //     return
                //   }
                //   if (e.key === 'Backspace') {
                //     e.preventDefault()
                //     if (viewmodel.slashPickerQuery.length <= 1) {
                //       viewmodel.closeSlashPicker()
                //       viewmodel.setSlashQuery('')
                //       return
                //     }
                //     viewmodel.setSlashQuery(viewmodel.slashPickerQuery.slice(0, -1))
                //     return
                //   }
                //   if (e.key.length === 1 && !e.metaKey && !e.ctrlKey && !e.altKey) {
                //     e.preventDefault()
                //     viewmodel.setSlashQuery(`${viewmodel.slashPickerQuery}${e.key}`)
                //     return
                //   }
                //   return
                // }
                // if (e.key === '/' && viewmodel.prompt.length === 0 && !viewmodel.isSlashOpen) {
                //   e.preventDefault()
                //   viewmodel.openSlashPicker()
                //   return
                // }
                if (e.key === 'Enter' && !e.shiftKey && !viewmodel.isSlashOpen) {
                  e.preventDefault()
                  handleSubmit(e)
                }
              }}
            />
          </div>

          {viewmodel.isRunning && (
            <div className="absolute bottom-1 right-2">
              <TooltipMacro withDelay tooltip="Stop the current task">
                <Button
                  className="size-[38px] relative hover:opacity-70 hover:[&_svg]:text-destructive"
                  size="icon"
                  variant="link"
                  onClick={handleStop}
                >
                  <Pause className="text-warn animate-pulse transition-colors duration-300" />
                  <LoaderCircleIcon className="absolute text-success/80 animate-spin size-9 transition-colors duration-300" />
                </Button>
              </TooltipMacro>
            </div>
          )}
        </form>
      </motion.div>
    </AnimatePresence>
  )
}

const toBase64 = async (blob: Blob) =>
  new Promise<string>((resolve) => {
    const reader = new FileReader()
    reader.onload = () => resolve(reader.result as string)
    reader.readAsDataURL(blob)
  })
