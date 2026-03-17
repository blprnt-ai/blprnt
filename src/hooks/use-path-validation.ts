import { stat } from '@tauri-apps/plugin-fs'
import { useEffect, useState } from 'react'
import { reportError } from '@/lib/utils/error-reporting'

interface PathValidation {
  isValid: boolean
  error?: string
  isChecking: boolean
}

export const usePathValidation = (path: string, debounceMs: number = 500): PathValidation => {
  const [validation, setValidation] = useState<PathValidation>({
    isChecking: false,
    isValid: true,
  })

  useEffect(() => {
    if (!path || path.trim() === '') {
      setValidation({ isChecking: false, isValid: false })
      return
    }

    setValidation((prev) => ({ ...prev, isChecking: true }))

    const timeoutId = setTimeout(async () => {
      try {
        const fileInfo = await stat(path)

        if (!fileInfo.isDirectory) {
          setValidation({
            error: 'Path must be a directory',
            isChecking: false,
            isValid: false,
          })
          return
        }

        setValidation({ isChecking: false, isValid: true })
      } catch (error) {
        reportError(error, 'validating path')
        setValidation({
          error: 'Path does not exist',
          isChecking: false,
          isValid: false,
        })
      }
    }, debounceMs)

    return () => clearTimeout(timeoutId)
  }, [path, debounceMs])

  return validation
}
