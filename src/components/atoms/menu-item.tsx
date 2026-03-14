import type { Content, Separator } from '@radix-ui/react-context-menu'
import type React from 'react'
import { cn } from '@/lib/utils/cn'

export const MenuContent = ({ className, children, ...props }: React.ComponentProps<typeof Content>) => {
  return (
    <div
      className={cn(
        'bg-popover text-popover-foreground',
        'data-[side=bottom]:slide-in-from-top-2',
        'data-[side=left]:slide-in-from-right-2',
        'data-[side=right]:slide-in-from-left-2',
        'data-[side=top]:slide-in-from-bottom-2',
        'overflow-x-hidden overflow-y-auto z-50',
        'rounded-md border p-1 shadow-md',
        className,
      )}
      {...props}
    >
      {children}
    </div>
  )
}

export const MenuItem = ({
  className,
  inset,
  variant = 'default',
  disabled = false,
  children,
  ...props
}: React.ComponentProps<'div'> & {
  inset?: boolean
  variant?: 'default' | 'destructive'
  disabled?: boolean
}) => {
  return (
    <div
      data-variant={variant}
      className={cn(
        'cursor-pointer relative flex items-center gap-2 rounded-sm px-2 py-1.5 text-sm outline-hidden select-none',
        'hover:bg-primary hover:text-primary-foreground',
        'data-[variant=destructive]:text-destructive data-[variant=destructive]:hover:bg-destructive/10 data-[variant=destructive]:hover:text-destructive data-[variant=destructive]:*:[svg]:text-destructive!',
        'data-inset:pl-8 [&_svg]:pointer-events-none',
        'whitespace-nowrap',
        "[&_svg:not([class*='text-'])]:text-muted-foreground [&_svg]:shrink-0 [&_svg:not([class*='size-'])]:size-4",
        disabled && 'pointer-events-none opacity-50',
        className,
      )}
      {...props}
    >
      {children}
    </div>
  )
}

export const MenuSeparator = ({ className, ...props }: React.ComponentProps<typeof Separator>) => {
  return (
    <div className={cn('bg-border -mx-1 my-1 h-px', className)} data-slot="context-menu-separator" {...props}></div>
  )
}
