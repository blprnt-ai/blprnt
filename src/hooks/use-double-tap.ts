import { useCallback, useRef, useState } from 'react'
import { useHotkeys } from 'react-hotkeys-hook'

export enum DoubleTapState {
  Clear,
  Single,
  Double,
}

const DEFAULT_DELAY = 2500

export const useDoubleTap = (enabled: boolean, key: string = 'esc', delay: number = DEFAULT_DELAY) => {
  const timeoutRef = useRef<number | null>(null)
  const [state, setState] = useState(DoubleTapState.Clear)

  const handleKeyDown = useCallback(() => {
    if (!enabled) return

    if (timeoutRef.current) clearTimeout(timeoutRef.current)
    timeoutRef.current = setTimeout(() => setState(DoubleTapState.Clear), delay)

    setState((prev) => {
      if (prev === DoubleTapState.Clear) return DoubleTapState.Single
      if (prev === DoubleTapState.Single) return DoubleTapState.Double

      return prev
    })
  }, [delay, enabled])

  useHotkeys(key, () => handleKeyDown(), { enabled, enableOnFormTags: true }, [key, enabled])

  return state
}
