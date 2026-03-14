import { cn } from '@/lib/utils/cn'

interface BgGridProps {
  className?: string
}

export const BgGrid = ({ className }: BgGridProps) => (
  <>
    <div className={cn('bg-gradient-glow absolute inset-0 pointer-events-none select-none z-0', className)} />
    <div className={cn('bg-grid-2 absolute inset-0 pointer-events-none select-none z-0', className)} />
  </>
)
