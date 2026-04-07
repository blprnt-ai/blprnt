import dayjs from 'dayjs'
import relativeTime from 'dayjs/plugin/relativeTime'

dayjs.extend(relativeTime)

export const formatRelativeTime = (value: Date, now?: Date) => {
  if (Number.isNaN(value.getTime())) return 'Unknown'
  return dayjs(value).from(now ? dayjs(now) : undefined)
}
