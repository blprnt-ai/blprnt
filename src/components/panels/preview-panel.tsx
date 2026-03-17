import { listen } from '@tauri-apps/api/event'
import { RefreshCcw, Square, TriangleAlert } from 'lucide-react'
import { useCallback, useEffect, useMemo, useState } from 'react'
import type { PreviewDetectedServer } from '@/bindings'
import { Button } from '@/components/atoms/button'
import { Empty, EmptyContent, EmptyDescription, EmptyHeader, EmptyMedia, EmptyTitle } from '@/components/atoms/empty'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/atoms/select'
import { basicToast as toast } from '@/components/atoms/toaster'
import {
  type PreviewMode,
  type PreviewServerAction,
  type PreviewSessionStatus,
  tauriPreviewApi,
  // eslint-disable-next-line boundaries/element-types
} from '@/lib/api/tauri/preview.api'
import { ProjectModel } from '@/lib/models/project.model'
import { cn } from '@/lib/utils/cn'
import { reportError } from '@/lib/utils/error-reporting'

const statusStyles: Record<PreviewSessionStatus, { label: string; className: string }> = {
  error: { className: 'bg-destructive/40 text-destructive-foreground', label: 'Error' },
  ready: { className: 'bg-success/40 text-success', label: 'Ready' },
  starting: { className: 'bg-primary/40 text-primary-foreground', label: 'Starting' },
  stopped: { className: 'bg-muted text-muted-foreground', label: 'Stopped' },
}

type PreviewInstrumentationEvent = {
  sessionId: string
  projectId: string
  eventType: string
  payload: Record<string, unknown>
  timestamp?: string | null
}

interface PreviewPanelProps {
  projectId: string
}

export const PreviewPanel = ({ projectId }: PreviewPanelProps) => {
  const [project, setProject] = useState<ProjectModel | null>(null)
  const [mode, setMode] = useState<PreviewMode>('dev')
  const [status, setStatus] = useState<PreviewSessionStatus>('stopped')
  const [sessionUrl, setSessionUrl] = useState<string | null>(null)
  const [lastError, setLastError] = useState<string | null>(null)
  const [isLoading, setIsLoading] = useState(false)
  const [instrumentationLogs, setInstrumentationLogs] = useState<string[]>([])
  const [activeSessionId, setActiveSessionId] = useState<string | null>(null)
  const [serverAction, setServerAction] = useState<PreviewServerAction | null>(null)
  const [detectedServer, setDetectedServer] = useState<PreviewDetectedServer | null>(null)
  const [stopMessage, setStopMessage] = useState<string | null>(null)
  const [displayUrl, setDisplayUrl] = useState<string | null>(null)

  const statusView = useMemo(() => statusStyles[status], [status])

  const refreshStatus = useCallback(async () => {
    try {
      const result = await tauriPreviewApi.status(projectId)

      if (result.status === 'error') {
        throw result.error
      }

      const data = result.data

      setStatus(data.status)
      setLastError(data.last_error?.message ?? null)
      setServerAction(data.server_action ?? null)
      setDetectedServer(data.detected ?? null)
      if (data.url) {
        setDisplayUrl(data.url)
        setSessionUrl(data.status === 'stopped' ? null : data.url)
      }
      if (data.status === 'stopped') {
        setActiveSessionId(null)
        if (data.was_auto_started) {
          setStopMessage('Auto-started server stopped')
        } else {
          setStopMessage(null)
        }
      }
    } catch (error) {
      reportError(error, 'refreshing preview status')
      setLastError(String(error))
    }
  }, [projectId])

  const handleStart = useCallback(async () => {
    if (!project) return

    setIsLoading(true)
    setLastError(null)

    try {
      const result = await tauriPreviewApi.start({
        allowed_hosts: null,
        dev_server_url: null,
        mode,
        project_id: projectId,
        proxy_port: null,
        static_path: null,
        static_port: null,
      })

      if (result.status === 'error') throw result.error

      const data = result.data

      setStatus(data.status)
      setSessionUrl(data.url)
      setDisplayUrl(data.url)
      setActiveSessionId(data.id)
      setStopMessage(null)
    } catch (error) {
      reportError(error, 'starting preview')
      const message = String(error)
      setLastError(message)
      toast.error({ description: message, title: 'Failed to start preview' })
    } finally {
      setIsLoading(false)
    }
  }, [mode, project, projectId])

  const handleStop = useCallback(async () => {
    setIsLoading(true)
    setLastError(null)

    try {
      await tauriPreviewApi.stop(projectId)
      await refreshStatus()
      setSessionUrl(null)
      setActiveSessionId(null)
    } catch (error) {
      reportError(error, 'stopping preview')
      const message = String(error)
      setLastError(message)
      toast.error({ description: message, title: 'Failed to stop preview' })
    } finally {
      setIsLoading(false)
    }
  }, [projectId, refreshStatus])

  const handleReload = useCallback(async () => {
    if (status === 'stopped') return

    setIsLoading(true)
    setLastError(null)

    try {
      const result = await tauriPreviewApi.reload(projectId)

      if (result.status === 'error') throw result.error
      const data = result.data

      setStatus(data.status)
      setSessionUrl(data.url)
      setDisplayUrl(data.url)
      setActiveSessionId(data.id)
      setStopMessage(null)
    } catch (error) {
      reportError(error, 'reloading preview')
      const message = String(error)
      setLastError(message)
      toast.error({ description: message, title: 'Failed to reload preview' })
    } finally {
      setIsLoading(false)
    }
  }, [projectId, status])

  useEffect(() => {
    refreshStatus().catch(() => {})
  }, [refreshStatus])

  useEffect(() => {
    let isMounted = true
    ProjectModel.get(projectId)
      .then((model) => {
        if (!isMounted) return
        setProject(model)
      })
      .catch((error) => {
        console.error('Error loading project', error)
      })

    return () => {
      isMounted = false
    }
  }, [projectId])

  useEffect(() => {
    if (status !== 'stopped' && !sessionUrl) {
      handleReload().catch(() => {})
    }
  }, [handleReload, sessionUrl, status])

  useEffect(() => {
    const interval = setInterval(() => {
      refreshStatus().catch(() => {})
    }, 5000)

    return () => clearInterval(interval)
  }, [refreshStatus])

  useEffect(() => {
    let unlisten: (() => void) | null = null
    listen<PreviewInstrumentationEvent>('previewInstrumentation', (event) => {
      if (event.payload.projectId !== projectId) return
      if (activeSessionId && event.payload.sessionId !== activeSessionId) return
      const timestamp = event.payload.timestamp ? ` [${event.payload.timestamp}]` : ''
      const line = `${event.payload.eventType}${timestamp}: ${JSON.stringify(event.payload.payload)}`
      setInstrumentationLogs((prev) => [...prev.slice(-49), line])
    })
      .then((stop) => {
        unlisten = stop
      })
      .catch(() => {})

    return () => {
      if (unlisten) {
        unlisten()
      }
    }
  }, [activeSessionId, projectId])

  useEffect(() => {
    if (status === 'stopped') {
      setInstrumentationLogs([])
    }
  }, [status])

  useEffect(() => {
    if (status !== 'stopped') {
      setStopMessage(null)
    }
  }, [status])

  const statusDetails = useMemo(() => {
    const details: Array<{ label: string; value: string }> = []

    if (serverAction) {
      details.push({ label: 'Server', value: serverAction === 'started' ? 'Auto-started' : 'Attached' })
    }

    if (detectedServer?.framework || detectedServer?.language) {
      const framework = detectedServer?.framework
      const language = detectedServer?.language
      const label = framework ? `Framework (${language ?? 'Unknown'})` : 'Language'
      const value = framework ?? language ?? 'Unknown'
      details.push({ label, value })
    }

    if (detectedServer?.suggested_port) {
      details.push({ label: 'Suggested port', value: `${detectedServer.suggested_port}` })
    }

    if (detectedServer?.detected_command) {
      details.push({ label: 'Detected command', value: detectedServer.detected_command })
    }

    if (displayUrl) {
      details.push({ label: 'URL', value: displayUrl })
    }

    return details
  }, [detectedServer, displayUrl, serverAction])

  if (!project) return null

  return (
    <div className="flex h-full w-full flex-col gap-4 p-4">
      <header className="flex items-center justify-between gap-4">
        <div className="flex flex-col gap-1">
          <div className="text-lg font-semibold">Preview</div>
          <div className="text-sm text-muted-foreground">{project.name}</div>
        </div>
        <div className="flex items-center gap-2">
          <StatusBadge className={statusView.className}>{statusView.label}</StatusBadge>
          <Select value={mode} onValueChange={(value) => setMode(value as PreviewMode)}>
            <SelectTrigger className="w-32" size="sm">
              <SelectValue placeholder="Select mode">{mode === 'dev' ? 'Dev' : 'Static'}</SelectValue>
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="dev">Dev</SelectItem>
              <SelectItem value="static">Static</SelectItem>
            </SelectContent>
          </Select>
        </div>
      </header>

      <div className="flex flex-col gap-3 rounded-md border bg-background-2/40 p-3 text-sm">
        {statusDetails.length > 0 ? (
          <div className="grid gap-2 md:grid-cols-2">
            {statusDetails.map((detail) => (
              <div key={detail.label} className="flex flex-col gap-1">
                <span className="text-xs uppercase text-muted-foreground">{detail.label}</span>
                <span className="font-medium text-foreground break-all">{detail.value}</span>
              </div>
            ))}
          </div>
        ) : (
          <div className="text-sm text-muted-foreground">No preview details available yet.</div>
        )}
        {stopMessage && <div className="text-xs text-muted-foreground">{stopMessage}</div>}
      </div>

      <div className="flex items-center gap-2">
        <Button disabled={isLoading} size="sm" onClick={handleStart}>
          Start
        </Button>
        <Button disabled={isLoading || status === 'stopped'} size="sm" variant="secondary" onClick={handleStop}>
          <Square className="size-3.5" /> Stop
        </Button>
        <Button disabled={isLoading || status === 'stopped'} size="sm" variant="outline" onClick={handleReload}>
          <RefreshCcw className="size-3.5" /> Reload
        </Button>
      </div>

      {lastError && (
        <div className="flex items-start gap-2 rounded-md border border-destructive/40 bg-destructive/10 p-3 text-sm">
          <TriangleAlert className="mt-0.5 size-4 text-destructive" />
          <div className="text-destructive">{lastError}</div>
        </div>
      )}

      <div className="flex-1 min-h-0 rounded-md border bg-background-2/40 overflow-hidden">
        {sessionUrl ? (
          <iframe className="h-full w-full" src={sessionUrl} title="Preview" />
        ) : (
          <Empty className="h-full">
            <EmptyHeader>
              <EmptyMedia variant="icon">
                <TriangleAlert className="size-5" />
              </EmptyMedia>
              <EmptyTitle>No preview running</EmptyTitle>
              <EmptyDescription>Start the preview to load the project webview.</EmptyDescription>
            </EmptyHeader>
            <EmptyContent>
              <Button disabled={isLoading} size="sm" onClick={handleStart}>
                Start preview
              </Button>
            </EmptyContent>
          </Empty>
        )}
      </div>

      {instrumentationLogs.length > 0 && (
        <div className="rounded-md border bg-background-2/60 p-3 text-xs text-muted-foreground">
          <div className="mb-2 text-sm font-medium text-foreground">Preview instrumentation</div>
          <div className="max-h-32 overflow-auto space-y-1 font-mono">
            {instrumentationLogs.map((entry, index) => (
              <div key={`${index}-${entry}`}>{entry}</div>
            ))}
          </div>
        </div>
      )}
    </div>
  )
}

const StatusBadge = ({ className, ...props }: React.ComponentProps<'div'>) => {
  return (
    <div
      className={cn(
        'rounded-full px-2.5 py-1 text-xs font-medium uppercase tracking-wide border border-transparent',
        className,
      )}
      {...props}
    />
  )
}
