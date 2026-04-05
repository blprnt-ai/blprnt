import { cn } from '@/lib/utils'

interface MetadataRowProps {
  label: string
  value: React.ReactNode
  className?: string
  labelClassName?: string
  valueClassName?: string
}

export const MetadataRow = ({ label, value, className, labelClassName, valueClassName }: MetadataRowProps) => {
  return (
    <div className={cn('flex items-start gap-3', className)}>
      <div className="min-w-0">
        <div className={cn('text-xs uppercase tracking-[0.18em] text-muted-foreground/50', labelClassName)}>
          {label}
        </div>
        <div className={cn('mt-1 wrap-break-word text-sm font-medium text-muted-foreground/90', valueClassName)}>
          {value}
        </div>
      </div>
    </div>
  )
}
