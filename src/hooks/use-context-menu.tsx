import { useCallback } from 'react'
import { useBlprntContextMenu } from './use-blprnt-context-menu'

export const useContextMenu = () => {
  const openMenu = useBlprntContextMenu()

  const handleOpenMenu = useCallback(
    async (e: React.MouseEvent) => {
      e.preventDefault()
      e.stopPropagation()
      openMenu(e, [])
    },
    [openMenu],
  )

  return handleOpenMenu
}
