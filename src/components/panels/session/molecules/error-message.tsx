import type { Dayjs } from 'dayjs'
import type { ErrorEvent } from '@/bindings'
import { RelativeTime } from '@/components/molecules/relative-time'
import { useIsDev } from '@/hooks/use-is-dev'
import { CATEGORY_LABELS, ERROR_MESSAGES } from '@/lib/utils/error'

export const ErrorMessage = ({ error, createdAt }: { error: ErrorEvent; createdAt: Dayjs }) => {
  const isDev = useIsDev()
  const errorInfo = ERROR_MESSAGES[error.code]
  const categoryLabel = CATEGORY_LABELS[error.category] ?? 'Error'

  return (
    <div className="flex flex-col gap-2 w-full">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2">
          <span className="text-xs font-medium uppercase tracking-wide">
            <span className="text-destructive">Error: </span>
            {categoryLabel}
          </span>
          {error.recoverable && (
            <span className="text-xs text-muted-foreground bg-muted px-1.5 py-0.5 rounded">Recoverable</span>
          )}
        </div>
        <RelativeTime timestamp={createdAt} />
      </div>
      <div className="font-medium">{errorInfo?.title ?? 'Error'}</div>
      <div className="text-muted-foreground">{errorInfo?.description ?? error.message}</div>
      {errorInfo && error.message && isDev && (
        <details className="text-xs">
          <summary className="cursor-pointer text-muted-foreground/60 hover:text-muted-foreground">
            Technical details
          </summary>
          <pre className="mt-1 whitespace-pre-wrap text-muted-foreground/60 font-mono">{error.message}</pre>
        </details>
      )}
    </div>
  )
}

export const InfoMessage = ({ message, createdAt }: { message: string; createdAt: Dayjs }) => {
  return (
    <div className="flex flex-col gap-2 w-full">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2">
          <span className="text-xs font-medium uppercase tracking-wide">
            <span className="text-info">Info: </span>
          </span>
        </div>
        <RelativeTime timestamp={createdAt} />
      </div>

      <div className="text-muted-foreground">{message}</div>
    </div>
  )
}

export const WarningMessage = ({ message, createdAt }: { message: string; createdAt: Dayjs }) => {
  return (
    <div className="flex flex-col gap-2 w-full">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2">
          <span className="text-xs font-medium uppercase tracking-wide">
            <span className="text-warn">Warning: </span>
          </span>
        </div>
        <RelativeTime timestamp={createdAt} />
      </div>
      <div className="text-muted-foreground">{message}</div>
    </div>
  )
}
