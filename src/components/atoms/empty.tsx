import { cva, type VariantProps } from 'class-variance-authority'

import { cn } from '@/lib/utils/cn'

export const Empty = ({ className, ...props }: React.ComponentProps<'div'>) => {
  return (
    <div
      data-slot="empty"
      className={cn(
        'flex min-w-0 flex-1 flex-col items-center justify-center gap-6 rounded-lg border-dashed p-6 text-center text-balance md:p-12',
        className,
      )}
      {...props}
    />
  )
}

export const EmptyHeader = ({ className, ...props }: React.ComponentProps<'div'>) => {
  return (
    <div
      className={cn('flex max-w-sm flex-col items-center gap-2 text-center', className)}
      data-slot="empty-header"
      {...props}
    />
  )
}

const emptyMediaVariants = cva(
  'flex shrink-0 items-center justify-center mb-2 [&_svg]:pointer-events-none [&_svg]:shrink-0',
  {
    defaultVariants: {
      variant: 'default',
    },
    variants: {
      variant: {
        default: 'bg-transparent',
        icon: "bg-muted text-foreground flex size-10 shrink-0 items-center justify-center rounded-lg [&_svg:not([class*='size-'])]:size-6",
      },
    },
  },
)

export const EmptyMedia = ({
  className,
  variant = 'default',
  ...props
}: React.ComponentProps<'div'> & VariantProps<typeof emptyMediaVariants>) => {
  return (
    <div
      className={cn(emptyMediaVariants({ className, variant }))}
      data-slot="empty-icon"
      data-variant={variant}
      {...props}
    />
  )
}

export const EmptyTitle = ({ className, ...props }: React.ComponentProps<'div'>) => {
  return <div className={cn('text-lg font-medium tracking-tight', className)} data-slot="empty-title" {...props} />
}

export const EmptyDescription = ({ className, ...props }: React.ComponentProps<'p'>) => {
  return (
    <div
      data-slot="empty-description"
      className={cn(
        'text-muted-foreground [&>a:hover]:text-primary text-sm/relaxed [&>a]:underline [&>a]:underline-offset-4',
        className,
      )}
      {...props}
    />
  )
}

export const EmptyContent = ({ className, ...props }: React.ComponentProps<'div'>) => {
  return (
    <div
      className={cn('flex w-full max-w-sm min-w-0 flex-col items-center gap-4 text-sm text-balance', className)}
      data-slot="empty-content"
      {...props}
    />
  )
}
