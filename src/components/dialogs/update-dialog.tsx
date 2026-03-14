import type { DownloadEvent, Update } from '@tauri-apps/plugin-updater'
import { useEffect, useMemo, useRef, useState } from 'react'
import { Button } from '@/components/atoms/button'
import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle } from '@/components/atoms/dialog'
import { Progress } from '@/components/atoms/progress'
import { UpdateState, useUpdateContext } from '@/hooks/use-update-context'

export interface UpdateDialogProps {
  canDismissDialog: boolean
  isOpen: boolean
  onOpenChange: (isOpen: boolean) => void
  update: Update
}

export const UpdateDialog = ({ canDismissDialog, isOpen, onOpenChange, update }: UpdateDialogProps) => {
  const chunkLength = useRef(1)
  const [downloadedBytes, setDownloadedBytes] = useState(0)
  const { onDownloadEvent, downloadUpdate, installUpdate, restartApp, state } = useUpdateContext()
  const progress = useMemo(() => (downloadedBytes / chunkLength.current) * 100, [downloadedBytes])

  const handleDownloadEvent = (event: DownloadEvent) => {
    switch (event.event) {
      case 'Started':
        chunkLength.current = event.data.contentLength ?? 1
        break
      case 'Progress':
        setDownloadedBytes((state) => state + event.data.chunkLength)
        break
      case 'Finished':
        setDownloadedBytes(chunkLength.current)
        break
    }
  }

  // biome-ignore lint/correctness/useExhaustiveDependencies: don't want to re-register the callback
  useEffect(() => {
    onDownloadEvent(handleDownloadEvent)
  }, [onDownloadEvent])

  const handleOpenChange = (isOpen: boolean) => {
    if (!canDismissDialog) return
    onOpenChange(isOpen)
  }

  return (
    <Dialog open={isOpen} onOpenChange={handleOpenChange}>
      <DialogContent showCloseButton={canDismissDialog}>
        <DialogHeader>
          <DialogTitle>
            Update Available: v{update.version} ({update.date})
          </DialogTitle>
        </DialogHeader>
        <DialogDescription>
          <span>Current version: {update.currentVersion}</span>
        </DialogDescription>

        <div className="flex flex-col gap-2">
          {state === UpdateState.Idle && (
            <Button variant="outline" onClick={downloadUpdate}>
              Download
            </Button>
          )}
          {state === UpdateState.Downloading && <Progress value={progress} />}
          {state === UpdateState.Downloaded && (
            <div className="flex gap-2">
              <Button className="w-1/2" variant="ghost" onClick={() => onOpenChange(false)}>
                Cancel
              </Button>

              <Button className="w-1/2" variant="outline" onClick={installUpdate}>
                Install
              </Button>
            </div>
          )}
          {state === UpdateState.Installing && <div>Installing...</div>}
          {state === UpdateState.Installed && (
            <div className="flex gap-2">
              <Button className="w-1/2" variant="ghost" onClick={() => onOpenChange(false)}>
                Cancel
              </Button>
              <Button className="w-1/2" variant="destructive" onClick={restartApp}>
                Restart
              </Button>
            </div>
          )}
        </div>
      </DialogContent>
    </Dialog>
  )
}
