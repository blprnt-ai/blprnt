import { Avatar, AvatarFallback } from '@/components/ui/avatar'
import { cn } from '@/lib/utils'
import { employeeIconValueToIcon } from '../ui/employee-label'

type IdentitySize = 'xs' | 'sm' | 'default' | 'lg'

export interface IdentityProps {
  name: string
  icon: string
  size?: IdentitySize
  className?: string
}

const textSize: Record<IdentitySize, string> = {
  default: 'text-sm',
  lg: 'text-sm',
  sm: 'text-xs',
  xs: 'text-sm',
}

const getIcon = (icon: string) => employeeIconValueToIcon(icon)

export function Identity({ name, icon, size = 'default', className }: IdentityProps) {
  const Icon = getIcon(icon)

  return (
    <span
      className={cn(
        'inline-flex items-center gap-1.5',
        size === 'xs' ? 'gap-1' : 'gap-1.5',
        size === 'lg' && 'gap-2',
        className,
      )}
    >
      <Avatar className={size === 'xs' ? 'relative' : undefined} size={size}>
        <AvatarFallback>{Icon && <Icon className="size-4.5" />}</AvatarFallback>
      </Avatar>
      <span className={cn('truncate', textSize[size])}>{name}</span>
    </span>
  )
}
