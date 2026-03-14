import type { DownloadEvent } from '@tauri-apps/plugin-updater'
import { createContext, useContext } from 'react'

export enum UpdateState {
  Idle = 'idle',
  Downloading = 'downloading',
  Downloaded = 'downloaded',
  Installing = 'installing',
  Installed = 'installed',
}

interface UpdateContextType {
  hasUpdate: boolean
  state: UpdateState
  openDialog: () => void
  downloadUpdate: () => void
  installUpdate: () => void
  restartApp: () => void
  onDownloadEvent: (callback: (event: DownloadEvent) => void) => () => void
}

export const UpdateContext = createContext<UpdateContextType>({
  downloadUpdate: () => {},
  hasUpdate: false,
  installUpdate: () => {},
  onDownloadEvent: () => () => {},
  openDialog: () => {},
  restartApp: () => {},
  state: UpdateState.Idle,
})

export const useUpdateContext = () => useContext(UpdateContext)
