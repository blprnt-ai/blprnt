import type { Variant } from 'framer-motion'
import { createContext } from 'react'
export type DisclosureContextType = {
  open: boolean
  toggle: () => void
  variants?: { expanded: Variant; collapsed: Variant }
}

export const DisclosureContext = createContext<DisclosureContextType | undefined>(undefined)
