import { useCallback, useEffect, useState } from 'react'
import { ModifierKey, ModifierKeyContext } from '@/hooks/use-modifier-keys'

export const ModifierKeyProvider = ({ children }: { children: React.ReactNode }) => {
  const [modifierKeys, setModifierKeys] = useState<Set<ModifierKey>>(new Set())

  const addKey = useCallback((key: ModifierKey) => setModifierKeys((prev) => new Set([...prev, key])), [])
  const removeKey = useCallback(
    (key: ModifierKey) => setModifierKeys((prev) => new Set([...prev].filter((k) => k !== key))),
    [],
  )

  useEffect(() => {
    const downHandler = (e: KeyboardEvent) => {
      if (e.metaKey) addKey(ModifierKey.Meta)
      else if (e.ctrlKey) addKey(ModifierKey.Control)
      else if (e.altKey) addKey(ModifierKey.Alt)
      else if (e.shiftKey) addKey(ModifierKey.Shift)
    }

    const upHandler = (e: KeyboardEvent) => {
      if (e.metaKey) removeKey(ModifierKey.Meta)
      else if (e.ctrlKey) removeKey(ModifierKey.Control)
      else if (e.altKey) removeKey(ModifierKey.Alt)
      else if (e.shiftKey) removeKey(ModifierKey.Shift)
    }

    document.addEventListener('keydown', downHandler)
    document.addEventListener('keyup', upHandler)

    return () => {
      document.removeEventListener('keydown', downHandler)
      document.removeEventListener('keyup', upHandler)
    }
  }, [addKey, removeKey])

  return <ModifierKeyContext.Provider value={{ modifierKeys }}>{children}</ModifierKeyContext.Provider>
}
