import { cn } from '@/lib/utils/cn'

type Props = {
  children: React.ReactNode
  className: string
}

export const TextSweep = ({ children, className }: Props) => (
  <div className={cn('text-sweep', className)}>{children}</div>
)
