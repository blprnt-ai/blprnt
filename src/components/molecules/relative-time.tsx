import type { Dayjs } from 'dayjs'
import { useRelativeTime } from '@/hooks/use-relative-time'
import { cn } from '@/lib/utils/cn'

interface RelativeTimeProps {
  timestamp: Dayjs
  className?: string
  as?: 'span' | 'time' | 'div'
}

export const RelativeTime = ({ timestamp, className, as: Component = 'span' }: RelativeTimeProps) => {
  const relativeTime = useRelativeTime(timestamp)

  return <Component className={cn('text-xs text-muted-foreground', className)}>{relativeTime}</Component>
}
