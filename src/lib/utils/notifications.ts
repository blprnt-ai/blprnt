import { isPermissionGranted, requestPermission, sendNotification } from '@tauri-apps/plugin-notification'

let permissionRequest: Promise<boolean> | null = null

const ensurePermission = async () => {
  if (permissionRequest) return permissionRequest

  permissionRequest = (async () => {
    const granted = await isPermissionGranted()
    if (granted) return true

    const permission = await requestPermission()
    return permission === 'granted'
  })()

  return permissionRequest
}

export const notify = async (title: string, body: string) => {
  const granted = await ensurePermission()
  if (!granted) return

  sendNotification({ body, title })
}
