import { cn } from '@/lib/utils'

export const IssueBadge = ({ children, className }: { children: React.ReactNode; className?: string }) => {
  return (
    <span className={cn('rounded-full border border-border/60 bg-muted/30 px-2.5 py-1 text-[11px] font-medium', className)}>
      {children}
    </span>
  )
}
