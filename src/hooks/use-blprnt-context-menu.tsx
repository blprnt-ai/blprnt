import {
  type CheckMenuItem,
  type CheckMenuItemOptions,
  type IconMenuItem,
  type IconMenuItemOptions,
  Menu,
  type MenuItem,
  type MenuItemOptions,
  type PredefinedMenuItem,
  type PredefinedMenuItemOptions,
  type Submenu,
  type SubmenuOptions,
} from '@tauri-apps/api/menu'
import { useCallback } from 'react'
import {
  getDevOnlyMenuItem,
  getOpenDevtoolsMenuItem,
  getReloadWindowMenuItem,
  getSeparatorMenuItem,
} from '@/lib/utils/menu-items-cache'
import { useIsDev } from './use-is-dev'

export type BlprntMenuItem =
  | Submenu
  | MenuItem
  | PredefinedMenuItem
  | CheckMenuItem
  | IconMenuItem
  | MenuItemOptions
  | SubmenuOptions
  | IconMenuItemOptions
  | PredefinedMenuItemOptions
  | CheckMenuItemOptions

export type MenuItems = BlprntMenuItem[]

export const useBlprntContextMenu = () => {
  const isDev = useIsDev()

  const handleCreateMenu = useCallback(
    async (initItems: MenuItems) => {
      return Menu.new({
        items: [
          ...initItems,
          ...(isDev
            ? [
                await getSeparatorMenuItem(),
                await getDevOnlyMenuItem(),
                await getSeparatorMenuItem(),
                await getReloadWindowMenuItem(),
                await getOpenDevtoolsMenuItem(),
              ]
            : []),
        ],
      })
    },
    [isDev],
  )

  return async (e: React.MouseEvent, initItems: MenuItems) => {
    e.preventDefault()
    e.stopPropagation()
    const menu = await handleCreateMenu(initItems)
    const menuItems = await menu?.items()

    if (!menuItems?.length) return

    menu.popup()
  }
}
