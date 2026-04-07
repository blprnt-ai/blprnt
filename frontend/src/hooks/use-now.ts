import { useEffect, useState } from 'react'

export const useNow = (intervalMs = 1000) => {
  const [now, setNow] = useState(() => new Date())

  useEffect(() => {
    const interval = window.setInterval(() => {
      setNow(new Date())
    }, intervalMs)

    return () => window.clearInterval(interval)
  }, [intervalMs])

  return now
}
