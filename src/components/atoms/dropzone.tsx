import { type Event, listen, TauriEvent } from '@tauri-apps/api/event'
import { AlertCircleIcon, UploadIcon } from 'lucide-react'
import { useEffect, useState } from 'react'
import { cn } from '@/lib/utils/cn'

const checkForValidExtensions = (paths: string[], extensions: string[]) =>
  paths.every((path) => extensions.some((extension) => path.endsWith(extension)))

export type DropzoneProps = {
  onDrop: (paths: string[]) => void
  accepts?: string[]
  className?: string
  maxItems?: number
  label?: string
}

interface DragEnterEvent {
  paths: string[]
  position: {
    x: number
    y: number
  }
}

type DragDropEvent = DragEnterEvent

export const Dropzone = ({
  onDrop,
  accepts,
  className,
  maxItems = 1,
  label = 'Drag and drop to upload',
}: DropzoneProps) => {
  const [isDragActive, setIsDragActive] = useState(false)
  const [isValid, setIsValid] = useState(true)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    const unlisten = listen(TauriEvent.DRAG_ENTER, (event: Event<DragEnterEvent>) => {
      if (event.payload.paths.length > maxItems) {
        setError(`You can only upload ${maxItems} file${maxItems > 1 ? 's' : ''}`)
        setIsValid(false)
        return
      }

      if (accepts && !checkForValidExtensions(event.payload.paths, accepts)) {
        setError(`You can only upload files with the following extensions: ${accepts.join(', ')}`)
        setIsValid(false)
        return
      }

      setIsDragActive(true)
      setIsValid(true)
      setError(null)
    })

    return () => {
      unlisten.then((unlistenFn) => unlistenFn())
    }
  }, [maxItems, accepts])

  useEffect(() => {
    const unlisten = listen(TauriEvent.DRAG_LEAVE, () => {
      setIsValid(true)
      setError(null)
      setIsDragActive(false)
    })

    return () => {
      unlisten.then((unlistenFn) => unlistenFn())
    }
  }, [])

  useEffect(() => {
    const unlisten = listen(TauriEvent.DRAG_DROP, (event: Event<DragDropEvent>) => {
      if (!isValid) return

      onDrop(event.payload.paths)
    })

    return () => {
      unlisten.then((unlistenFn) => unlistenFn())
    }
  }, [onDrop, isValid])

  return (
    <div
      className={cn(
        'flex flex-col items-center justify-center relative h-auto w-full overflow-hidden p-4 border border-dashed border-primary/60 rounded-md transition-colors',
        isDragActive && 'border-primary bg-white/10',
        !isValid && 'border-red-500 bg-red-500/10',
        className,
      )}
    >
      {isValid ? <DropzoneEmptyState label={label} /> : <DropzoneErrorState error={error!} />}
    </div>
  )
}

export type DropzoneErrorStateProps = {
  error: string
}

export const DropzoneErrorState = ({ error }: DropzoneErrorStateProps) => {
  return (
    <div className="flex flex-col items-center justify-center gap-2">
      <div className="flex size-8 items-center justify-center rounded-md bg-destructive/60 text-destructive-foreground">
        <AlertCircleIcon size={16} />
      </div>
      <p className="w-full truncate text-wrap text-muted-foreground text-xs">{error}</p>
    </div>
  )
}

export type DropzoneEmptyStateProps = {
  label: string
}

export const DropzoneEmptyState = ({ label }: DropzoneEmptyStateProps) => {
  return (
    <div className="flex flex-col items-center justify-center gap-2">
      <div className="flex size-8 items-center justify-center rounded-md bg-muted text-muted-foreground">
        <UploadIcon size={16} />
      </div>
      <p className="w-full truncate text-wrap text-muted-foreground text-xs">{label}</p>
    </div>
  )
}
