import { useContext } from 'react'
import { DisclosureContext } from '@/context/disclosure-context'

export const useDisclosure = () => {
  const context = useContext(DisclosureContext)
  if (!context) {
    throw new Error('useDisclosure must be used within a DisclosureProvider')
  }
  return context
}
