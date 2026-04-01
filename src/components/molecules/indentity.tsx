import { Link } from '@tanstack/react-router'
import { Avatar, AvatarFallback } from '@/components/ui/avatar'
import { cn } from '@/lib/utils'
import { type ColorVariant, TextColoredSpan } from '../ui/colors'
import { employeeIconValueToIcon } from '../ui/employee-label'

type IdentitySize = 'xs' | 'sm' | 'default' | 'lg'

export interface IdentityProps {
  employeeId: string | null
  name?: string
  icon: string
  color: ColorVariant
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

export function IdentityLink({ employeeId, ...props }: IdentityProps) {
  if (!employeeId) {
    return <Identity {...props} />
  }

  return (
    <Link params={{ employeeId }} to="/employees/$employeeId">
      <Identity {...props} />
    </Link>
  )
}

export const Identity = ({ name, icon, color, size = 'default', className }: Omit<IdentityProps, 'employeeId'>) => {
  const IconBase = getIcon(icon)
  const Icon = () => (
    <TextColoredSpan color={color}>
      <IconBase className="size-4.5" />
    </TextColoredSpan>
  )

  return (
    <span
      className={cn(
        'inline-flex items-center gap-1.5 hover:text-primary hover:underline transition-colors duration-300',
        size === 'xs' ? 'gap-1' : 'gap-1.5',
        size === 'lg' && 'gap-2',
        className,
      )}
    >
      <Avatar className={size === 'xs' ? 'relative' : undefined} size={size}>
        <AvatarFallback>{Icon && <Icon />}</AvatarFallback>
      </Avatar>
      {name && <span className={cn('truncate', textSize[size])}>{name}</span>}
    </span>
  )
}
