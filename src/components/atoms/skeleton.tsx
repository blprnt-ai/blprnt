import { cn } from '@/lib/utils/cn'

function Skeleton({ className, ...props }: React.ComponentProps<'div'>) {
  return <div className={cn('bg-accent rounded-md animate-pulse', className)} data-slot="skeleton" {...props} />
}

export { Skeleton }
