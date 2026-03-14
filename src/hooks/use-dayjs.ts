import type { Dayjs } from 'dayjs'
import { useMemo } from 'react'
import { toDayJs } from '@/lib/utils/misc'

export const useDayJs = (timestamp: string | number | Date | Dayjs) => {
  return useMemo(() => toDayJs(timestamp), [timestamp])
}
