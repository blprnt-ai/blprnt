import { platform } from '@tauri-apps/plugin-os'

export const isLinux = platform().toLowerCase() === 'linux'
