import { cn } from '@/lib/utils/cn'

export const AlertBox = ({
  title,
  description,
  variant,
}: {
  title?: string
  description: string
  variant: 'info' | 'warning' | 'danger' | 'success'
}) => {
  return (
    <div
      className={cn(
        'border  p-2 rounded-md flex gap-1 items-start justify-center',
        variant === 'info' && 'border-info text-info',
        variant === 'warning' && 'border-warn text-warn',
        variant === 'danger' && 'border-destructive text-destructive',
        variant === 'success' && 'border-success text-success',
      )}
    >
      {title && <span className="font-semibold whitespace-nowrap">{title}:</span>}
      <span className="font-normal">{description}</span>
    </div>
  )
}
