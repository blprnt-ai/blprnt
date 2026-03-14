import { MenuItem, PredefinedMenuItem } from '@tauri-apps/api/menu'
import type { BlprntMenuItem } from '@/hooks/use-blprnt-context-menu'
import { tauriCommandApi } from '@/lib/api/tauri/command.api'
import { queryClient } from './query-client'

export const getMenuItem = (key: string, queryFn: () => Promise<BlprntMenuItem>) => {
  return queryClient.fetchQuery({
    queryFn,
    queryKey: ['menu-items', key],
    staleTime: Infinity,
  })
}

// Predefined
export const getSeparatorMenuItem = () => getMenuItem('separator', () => PredefinedMenuItem.new({ item: 'Separator' }))
export const getCutMenuItem = () => getMenuItem('cut', () => PredefinedMenuItem.new({ item: 'Cut' }))
export const getCopyMenuItem = () => getMenuItem('copy', () => PredefinedMenuItem.new({ item: 'Copy' }))
export const getCutMenuItemDisabled = () =>
  getMenuItem('cut-disabled', () => MenuItem.new({ enabled: false, text: 'Cut' }))
export const getCopyMenuItemDisabled = () =>
  getMenuItem('copy-disabled', () => MenuItem.new({ enabled: false, text: 'Copy' }))
export const getPasteMenuItem = () => getMenuItem('paste', () => PredefinedMenuItem.new({ item: 'Paste' }))
export const getSelectAllMenuItem = () => getMenuItem('select-all', () => PredefinedMenuItem.new({ item: 'SelectAll' }))

// Dev
export const getDevOnlyMenuItem = () =>
  getMenuItem('dev-only', () => MenuItem.new({ enabled: false, text: 'DEV ONLY' }))
export const getReloadWindowMenuItem = () =>
  getMenuItem('reload-window', () => MenuItem.new({ action: () => tauriCommandApi.reload(), text: 'Reload Window' }))
export const getOpenDevtoolsMenuItem = () =>
  getMenuItem('open-devtools', () => MenuItem.new({ action: () => tauriCommandApi.devtools(), text: 'Open Devtools' }))
