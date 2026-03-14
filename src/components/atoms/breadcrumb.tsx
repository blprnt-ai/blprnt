import { Slot } from '@radix-ui/react-slot'
import { ChevronRight, MoreHorizontal } from 'lucide-react'
import type React from 'react'

import { cn } from '@/lib/utils/cn'

export const Breadcrumb = ({ ...props }: React.ComponentProps<'nav'>) => {
  return <nav aria-label="breadcrumb" data-slot="breadcrumb" {...props} />
}

export const BreadcrumbList = ({ className, ...props }: React.ComponentProps<'ol'>) => {
  return (
    <ol
      data-slot="breadcrumb-list"
      className={cn(
        'text-muted-foreground flex flex-wrap items-center gap-1.5 text-lg font-medium wrap-break-words sm:gap-2.5',
        className,
      )}
      {...props}
    />
  )
}

export const BreadcrumbItem = ({ className, ...props }: React.ComponentProps<'li'>) => {
  return (
    <li
      data-slot="breadcrumb-item"
      className={cn(
        'inline-flex items-center gap-1.5 not-last:text-primary/60 not-last:hover:underline underline-offset-2',
        className,
      )}
      {...props}
    />
  )
}

export const BreadcrumbLink = ({
  asChild,
  className,
  ...props
}: React.ComponentProps<'a'> & {
  asChild?: boolean
}) => {
  const Comp = asChild ? Slot : 'a'

  return (
    <Comp
      className={cn('hover:text-foreground transition-colors cursor-pointer', className)}
      data-slot="breadcrumb-link"
      {...props}
    />
  )
}

export const BreadcrumbPage = ({ className, ...props }: React.ComponentProps<'span'>) => {
  return (
    <span
      aria-current="page"
      aria-disabled="true"
      className={cn('text-foreground font-normal', className)}
      data-slot="breadcrumb-page"
      role="link"
      {...props}
    />
  )
}

export const BreadcrumbSeparator = ({ children, className, ...props }: React.ComponentProps<'li'>) => {
  return (
    <li
      aria-hidden="true"
      className={cn('[&>svg]:size-3.5', className)}
      data-slot="breadcrumb-separator"
      role="presentation"
      {...props}
    >
      {children ?? <ChevronRight />}
    </li>
  )
}

export const BreadcrumbEllipsis = ({ className, ...props }: React.ComponentProps<'span'>) => {
  return (
    <span
      aria-hidden="true"
      className={cn('flex size-9 items-center justify-center', className)}
      data-slot="breadcrumb-ellipsis"
      role="presentation"
      {...props}
    >
      <MoreHorizontal className="size-4" />
      <span className="sr-only">More</span>
    </span>
  )
}
