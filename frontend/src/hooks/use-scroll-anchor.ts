import { useCallback, useEffect, useMemo, useState } from 'react'

interface UseScrollAnchorOptions {
  bottomThreshold?: number
}

export const useScrollAnchor = ({ bottomThreshold = 150 }: UseScrollAnchorOptions = {}) => {
  const [container, setContainer] = useState<HTMLElement | null>(null)
  const [isNearBottom, setIsNearBottom] = useState(true)

  const updateScrollState = useCallback(() => {
    if (!container) return

    const distanceFromBottom = container.scrollHeight - container.scrollTop - container.clientHeight
    setIsNearBottom(distanceFromBottom <= bottomThreshold)
  }, [bottomThreshold, container])

  useEffect(() => {
    if (!container) return

    updateScrollState()
    container.addEventListener('scroll', updateScrollState, { passive: true })

    return () => {
      container.removeEventListener('scroll', updateScrollState)
    }
  }, [container, updateScrollState])

  const scrollToBottom = useCallback(
    (behavior: ScrollBehavior = 'smooth') => {
      if (!container) return

      container.scrollTo({
        behavior,
        top: container.scrollHeight,
      })
    },
    [container],
  )

  return useMemo(
    () => ({
      isNearBottom,
      scrollToBottom,
      setContainer,
      updateScrollState,
    }),
    [isNearBottom, scrollToBottom, updateScrollState],
  )
}
