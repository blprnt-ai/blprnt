import type { PropsWithChildren } from 'react'
import { cn } from '@/lib/utils/cn'

export const WarningBox = ({ children, className }: PropsWithChildren<{ className?: string }>) => {
  return (
    <div className={cn('text-warn border border-warn rounded-md px-4 py-3 w-fit', className)}>
      <div className="text-xs font-medium uppercase tracking-wide">Warning:</div>
      <div className="text-muted-foreground font-normal">{children}</div>
    </div>
  )
}

export const InfoBox = ({ children, className }: PropsWithChildren<{ className?: string }>) => {
  return (
    <div className={cn('text-info border border-info rounded-md px-4 py-3 w-fit', className)}>
      <div className="text-xs font-medium uppercase tracking-wide">Info:</div>
      <div className="text-muted-foreground font-normal">{children}</div>
    </div>
  )
}

export const PlainBox = ({ children, className }: PropsWithChildren<{ className?: string }>) => {
  return <div className={cn('border border-border/60 rounded-md px-4 py-3 w-fit bg-accent', className)}>{children}</div>
}
