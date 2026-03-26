import { Avatar, AvatarFallback, AvatarImage } from '@/components/ui/avatar'
import { cn } from '@/lib/utils'

type IdentitySize = 'xs' | 'sm' | 'default' | 'lg'

export interface IdentityProps {
  name: string
  avatarUrl?: string | null
  initials?: string
  size?: IdentitySize
  className?: string
}

function deriveInitials(name: string): string {
  const parts = name.trim().split(/\s+/)
  if (parts.length >= 2) return (parts[0][0] + parts[parts.length - 1][0]).toUpperCase()
  return name.slice(0, 2).toUpperCase()
}

const textSize: Record<IdentitySize, string> = {
  default: 'text-sm',
  lg: 'text-sm',
  sm: 'text-xs',
  xs: 'text-sm',
}

export function Identity({ name, avatarUrl, initials, size = 'default', className }: IdentityProps) {
  const displayInitials = initials ?? deriveInitials(name)

  return (
    <span
      className={cn(
        'inline-flex gap-1.5',
        size === 'xs' ? 'items-baseline gap-1' : 'items-center',
        size === 'lg' && 'gap-2',
        className,
      )}
    >
      <Avatar className={size === 'xs' ? 'relative -top-px' : undefined} size={size}>
        {avatarUrl && <AvatarImage alt={name} src={avatarUrl} />}
        <AvatarFallback>{displayInitials}</AvatarFallback>
      </Avatar>
      <span className={cn('truncate', textSize[size])}>{name}</span>
    </span>
  )
}
