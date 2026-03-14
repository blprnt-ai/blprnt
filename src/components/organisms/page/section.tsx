import type { PropsWithChildren } from 'react'
import { cn } from '@/lib/utils/cn'

interface SectionProps extends PropsWithChildren {
  className?: string
}

export const Section = ({ children, className }: PropsWithChildren<SectionProps>) => {
  return (
    <div className={cn('text-sm font-medium border-t border-border/25 w-full max-w-6xl', className)}>{children}</div>
  )
}
