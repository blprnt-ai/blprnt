import { createContext, useContext } from 'react'

export const TextSelectedContext = createContext<string>('')

export const useTextSelected = () => useContext(TextSelectedContext)
