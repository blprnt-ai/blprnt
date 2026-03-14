import { createContext, useContext } from 'react'

export enum ModifierKey {
  Meta = 'Meta',
  Control = 'Control',
  Alt = 'Alt',
  Shift = 'Shift',
}

export interface ModifierKeyContext {
  modifierKeys: Set<ModifierKey>
}

export const ModifierKeyContext = createContext<ModifierKeyContext>({
  modifierKeys: new Set(),
})

export const useModifierKeys = () => useContext(ModifierKeyContext)
