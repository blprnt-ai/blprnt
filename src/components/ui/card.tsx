import type * as React from 'react'

import { cn } from '@/lib/utils'

export const Card = ({
  className,
  size = 'default',
  ...props
}: React.ComponentProps<'div'> & { size?: 'default' | 'sm' }) => {
  return (
    <div
      data-size={size}
      data-slot="card"
      className={cn(
        'group/card flex flex-col gap-6 overflow-hidden rounded-sm bg-card py-6 text-sm text-card-foreground shadow-xs ring-1 ring-border/80 has-[>img:first-child]:pt-0 data-[size=sm]:gap-4 data-[size=sm]:py-4 *:[img:first-child]:rounded-t-md *:[img:last-child]:rounded-b-md',
        className,
      )}
      {...props}
    />
  )
}

export const CardHeader = ({ className, ...props }: React.ComponentProps<'div'>) => {
  return (
    <div
      data-slot="card-header"
      className={cn(
        'group/card-header @container/card-header grid auto-rows-min items-start gap-1 rounded-t-md px-6 group-data-[size=sm]/card:px-4 has-data-[slot=card-action]:grid-cols-[1fr_auto] has-data-[slot=card-description]:grid-rows-[auto_auto] [.border-b]:border-border/80 [.border-b]:pb-6 group-data-[size=sm]/card:[.border-b]:pb-4',
        className,
      )}
      {...props}
    />
  )
}

export const CardTitle = ({ className, ...props }: React.ComponentProps<'div'>) => {
  return (
    <div
      className={cn('font-heading text-base leading-normal font-medium text-foreground group-data-[size=sm]/card:text-sm', className)}
      data-slot="card-title"
      {...props}
    />
  )
}

export const CardDescription = ({ className, ...props }: React.ComponentProps<'div'>) => {
  return <div className={cn('text-sm text-muted-foreground', className)} data-slot="card-description" {...props} />
}

export const CardAction = ({ className, ...props }: React.ComponentProps<'div'>) => {
  return (
    <div
      className={cn('col-start-2 row-span-2 row-start-1 self-start justify-self-end', className)}
      data-slot="card-action"
      {...props}
    />
  )
}

export const CardContent = ({ className, ...props }: React.ComponentProps<'div'>) => {
  return <div className={cn('px-6 group-data-[size=sm]/card:px-4', className)} data-slot="card-content" {...props} />
}

export const CardFooter = ({ className, ...props }: React.ComponentProps<'div'>) => {
  return (
    <div
      data-slot="card-footer"
      className={cn(
        'flex items-center rounded-b-md px-6 group-data-[size=sm]/card:px-4 [.border-t]:border-border/80 [.border-t]:pt-6 group-data-[size=sm]/card:[.border-t]:pt-4',
        className,
      )}
      {...props}
    />
  )
}
