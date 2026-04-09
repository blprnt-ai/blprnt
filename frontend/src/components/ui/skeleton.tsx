import { cn } from '@/lib/utils'

export const Skeleton = ({ className, ...props }: React.ComponentProps<'div'>) => {
  return <div className={cn('animate-pulse rounded-md bg-accent/65', className)} data-slot="skeleton" {...props} />
}
