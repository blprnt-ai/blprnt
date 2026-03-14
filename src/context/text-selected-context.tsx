import { useEffect, useState } from 'react'
import { TextSelectedContext } from '@/hooks/use-text-selected'

export const TextSelectedProvider = ({ children }: { children: React.ReactNode }) => {
  const [textSelected, setTextSelected] = useState<string>('')

  useEffect(() => {
    const handleSelectionChange = () => {
      try {
        const selection = window.getSelection()
        const text = selection?.toString() ?? ''
        setTextSelected(text)
      } catch (error) {
        console.error('[TextSelectedProvider] Failed to get selection', error)
        console.error(error)
      }
    }

    document.addEventListener('selectionchange', handleSelectionChange)

    return () => document.removeEventListener('selectionchange', handleSelectionChange)
  }, [])

  return <TextSelectedContext.Provider value={textSelected}>{children}</TextSelectedContext.Provider>
}
