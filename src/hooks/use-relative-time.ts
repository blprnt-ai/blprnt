import type { Dayjs } from 'dayjs'
import { useEffect, useState } from 'react'
import { nextRelativeTick, toHumanTime } from '@/lib/utils/misc'
import { useDayJs } from './use-dayjs'

export const useRelativeTime = (timestamp: Dayjs) => {
  const dayjs = useDayJs(timestamp)
  const [relativeTime, setRelativeTime] = useState(toHumanTime(dayjs))

  useEffect(() => {
    let timeout: ReturnType<typeof setTimeout> | null = null
    let cancelled = false

    setRelativeTime(toHumanTime(dayjs))

    const scheduleNextUpdate = () => {
      const nextTick = nextRelativeTick(dayjs)

      timeout = setTimeout(() => {
        if (cancelled) return
        setRelativeTime(toHumanTime(dayjs))
        scheduleNextUpdate()
      }, nextTick)
    }

    scheduleNextUpdate()

    return () => {
      cancelled = true
      if (timeout) clearTimeout(timeout)
    }
  }, [dayjs])

  return relativeTime
}
