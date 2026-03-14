import { useEffect, useState } from 'react'

export const useTextTruncated = (ref: React.RefObject<HTMLElement | null>) => {
  const [isTruncated, setIsTruncated] = useState(false)

  useEffect(() => {
    if (!ref.current) return

    setIsTruncated(
      ref.current.scrollWidth > ref.current.clientWidth || ref.current.scrollHeight > ref.current.clientHeight,
    )
  }, [ref.current])

  return isTruncated
}
