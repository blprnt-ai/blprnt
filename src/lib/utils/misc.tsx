import { captureException } from '@sentry/react'
import dayjs, { type Dayjs } from 'dayjs'
import relativeTime from 'dayjs/plugin/relativeTime'
import utc from 'dayjs/plugin/utc'
import type { ErrorEvent, TauriError } from '@/bindings'
import type { ToastProps } from '@/components/atoms/toast'
import { ERROR_MESSAGES } from './error'

dayjs.extend(relativeTime)
dayjs.extend(utc)

export const toDayJs = (timestamp: string | number | Date | Dayjs) => {
  if (typeof timestamp === 'string') return dayjs.utc(timestamp)
  if (typeof timestamp === 'number') return dayjs.utc(timestamp * 1000)
  if (timestamp && typeof timestamp === 'object' && 'fromNow' in timestamp) return timestamp

  return dayjs.utc(timestamp)
}

export const toHumanTime = (timestamp: string | number | Date | Dayjs | undefined) => {
  if (typeof timestamp === 'string' && !Number.isNaN(Number(timestamp)))
    return dayjs.utc(Number(timestamp) * 1000 || 0).fromNow()
  else if (typeof timestamp === 'string') return dayjs.utc(timestamp).fromNow()
  else if (typeof timestamp === 'number') return dayjs.utc(Number(timestamp) * 1000 || 0).fromNow()
  else if (timestamp instanceof Date) return dayjs.utc(timestamp).fromNow()
  else if (timestamp && typeof timestamp === 'object' && 'fromNow' in timestamp) return timestamp.fromNow()

  return undefined
}

export const nextRelativeTick = (target: Dayjs) => {
  const diff = Math.abs(dayjs().diff(target))

  if (diff < 60_000) return 1_000 - (diff % 1_000) || 1_000
  if (diff < 3_600_000) return 60_000 - (diff % 60_000) || 60_000
  if (diff < 86_400_000) return 3_600_000 - (diff % 3_600_000) || 3_600_000

  return 86_400_000 - (diff % 86_400_000) || 86_400_000
}

export const asyncWait = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms))
export const randomWait = (min: number, max: number) => asyncWait(Math.floor(Math.random() * (max - min + 1)) + min)
export const waitForAction = () => randomWait(100, 400)

export const hashCode = (str: string, seed = 0) => {
  let h1 = 0xdeadbeef ^ seed,
    h2 = 0x41c6ce57 ^ seed
  for (let i = 0, ch; i < str.length; i++) {
    ch = str.charCodeAt(i)
    h1 = Math.imul(h1 ^ ch, 2654435761)
    h2 = Math.imul(h2 ^ ch, 1597334677)
  }
  h1 = Math.imul(h1 ^ (h1 >>> 16), 2246822507)
  h2 = Math.imul(h2 ^ (h2 >>> 13), 3266489909)
  h1 ^= Math.imul(h2 ^ (h2 >>> 16), 3266489909)
  h2 ^= Math.imul(h1 ^ (h1 >>> 13), 2246822507)

  return 4294967296 * (2097151 & h2) + (h1 >>> 0)
}

export const errorToMessage = (tauriError: TauriError, logError: boolean = false): ToastProps => {
  if (logError) captureException(tauriError)

  if (!tauriError.error) {
    return {
      description: tauriError.message,
      size: 'wide',
      title: 'Error',
    }
  } else {
    const error = tauriError.error as ErrorEvent

    const title = ERROR_MESSAGES[error.code] ? ERROR_MESSAGES[error.code].title : 'Error'
    const message = ERROR_MESSAGES[error.code] ? ERROR_MESSAGES[error.code].description : error.message

    const description = (
      <div className="flex flex-col gap-4">
        <pre className="whitespace-pre-wrap font-base font-sans">{message}</pre>
      </div>
    )

    return {
      description,
      size: 'wide',
      title,
    }
  }
}

export const isTextTruncated = (element: HTMLElement) =>
  element.offsetWidth < element.scrollWidth || element.offsetHeight < element.scrollHeight
