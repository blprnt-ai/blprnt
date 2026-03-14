import { type ClassValue, clsx } from 'clsx'
import dayjs from 'dayjs'
import relativeTime from 'dayjs/plugin/relativeTime'
import utc from 'dayjs/plugin/utc'
import { twMerge } from 'tailwind-merge'

dayjs.extend(relativeTime)
dayjs.extend(utc)

export const cn = (...inputs: ClassValue[]) => {
  return twMerge(clsx(inputs))
}
