import { captureException } from '@sentry/react'
import { relaunch } from '@tauri-apps/plugin-process'
import { check, type DownloadEvent, type Update } from '@tauri-apps/plugin-updater'
import { useCallback, useEffect, useMemo, useRef, useState } from 'react'
import { updateToast as toast } from '@/components/atoms/toaster'
import { UpdateDialog } from '@/components/dialogs/update-dialog'
import { useAppViewModel } from '@/hooks/use-app-viewmodel'
import { UpdateContext, UpdateState } from '@/hooks/use-update-context'

const THIRTY_MINUTES = 1000 * 60 * 30
export const UpdateProvider = ({ children }: { children: React.ReactNode }) => {
  const { bannerModel } = useAppViewModel()
  const [isDialogOpen, setIsDialogOpen] = useState(false)
  const [canDismissDialog, setCanDismissDialog] = useState(true)

  const [state, setState] = useState(UpdateState.Idle)
  const downloadEventCallbacks = useRef<Set<(event: DownloadEvent) => void>>(new Set())

  const [update, setUpdate] = useState<Update | null>(null)
  const hasUpdate = useMemo(() => update !== null, [update])

  const handleBannerAction = useCallback(() => {
    setIsDialogOpen(true)
  }, [])

  useEffect(() => {
    bannerModel.setAction(handleBannerAction)
  }, [bannerModel, handleBannerAction])

  const handleNewUpdate = useCallback(
    (update: Update | null) => {
      if (!update) return

      bannerModel.setContent(`Update available: ${update.version}`)
      bannerModel.setShowBanner(true)
      setUpdate(update)
    },
    [bannerModel],
  )

  useEffect(() => {
    checkForUpdate(handleNewUpdate)

    const interval = setInterval(() => checkForUpdate(setUpdate), THIRTY_MINUTES)

    return () => clearInterval(interval)
  }, [handleNewUpdate])

  const handleDownloadEvent = useCallback((event: DownloadEvent) => {
    for (const callback of Array.from(downloadEventCallbacks.current)) {
      callback(event)

      if (event.event === 'Finished') {
        toast.success({ dismissible: false, duration: 2500, title: 'Update downloaded' })
        setState(UpdateState.Downloaded)
      }
    }
  }, [])

  const handleDownloadUpdate = useCallback(async () => {
    if (!update) return
    toast.loading({ dismissible: false, title: 'Downloading update...' })
    setCanDismissDialog(false)
    setState(UpdateState.Downloading)

    await downloadUpdate(update, handleDownloadEvent)
  }, [update, handleDownloadEvent])

  const handleInstallUpdate = useCallback(async () => {
    if (!update) return
    toast.loading({ dismissible: false, title: 'Installing update...' })
    setState(UpdateState.Installing)

    await installUpdate(update)
    toast.success({ dismissible: false, duration: 2500, title: 'Update installed' })
    setCanDismissDialog(true)

    setState(UpdateState.Installed)
  }, [update])

  const handleRestartApp = useCallback(async () => {
    if (!update) return

    await restartApp()
  }, [update])

  const registerDownloadEventCallback = useCallback((callback: (event: DownloadEvent) => void) => {
    downloadEventCallbacks.current.add(callback)

    return () => {
      downloadEventCallbacks.current.delete(callback)
    }
  }, [])

  const providerValue = useMemo(
    () => ({
      downloadUpdate: handleDownloadUpdate,
      hasUpdate,
      installUpdate: handleInstallUpdate,
      onDownloadEvent: registerDownloadEventCallback,
      openDialog: () => setIsDialogOpen(true),
      restartApp: handleRestartApp,
      state,
    }),
    [handleDownloadUpdate, hasUpdate, handleInstallUpdate, registerDownloadEventCallback, handleRestartApp, state],
  )

  return (
    <UpdateContext.Provider value={providerValue}>
      {children}
      {isDialogOpen && update && (
        <UpdateDialog
          canDismissDialog={canDismissDialog}
          isOpen={isDialogOpen}
          update={update}
          onOpenChange={setIsDialogOpen}
        />
      )}
    </UpdateContext.Provider>
  )
}

const checkForUpdate = async (callback: (update: Update | null) => void) => {
  try {
    const update = await check()

    callback(update)
  } catch (error) {
    captureException(error)
    // console.error('Error checking for updates:', error)
  }
}

const downloadUpdate = async (update: Update, onEvent: (event: DownloadEvent) => void) => {
  try {
    return await update.download(onEvent)
  } catch (error) {
    captureException(error)
    console.error('Error installing update:', error)
  }
}

const installUpdate = async (update: Update) => {
  try {
    return await update.install()
  } catch (error) {
    captureException(error)
    console.error('Error installing update:', error)
  }
}

const restartApp = async () => {
  try {
    return await relaunch()
  } catch (error) {
    captureException(error)
    console.error('Error restarting app:', error)
  }
}
