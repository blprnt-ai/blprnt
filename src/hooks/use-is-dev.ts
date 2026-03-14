import { useMemo } from 'react'

export const useIsDev = () => useMemo(() => import.meta.env.DEV, [])
