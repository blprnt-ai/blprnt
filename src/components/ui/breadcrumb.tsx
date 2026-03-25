import { mergeProps } from '@base-ui/react/merge-props'
import { useRender } from '@base-ui/react/use-render'
import { ChevronRightIcon, MoreHorizontalIcon } from 'lucide-react'
import type * as React from 'react'
import { cn } from '@/lib/utils'

export const Breadcrumb = ({ className, ...props }: React.ComponentProps<'nav'>) => {
  return <nav aria-label="breadcrumb" className={cn(className)} data-slot="breadcrumb" {...props} />
}

export const BreadcrumbList = ({ className, ...props }: React.ComponentProps<'ol'>) => {
  return (
    <ol
      data-slot="breadcrumb-list"
      className={cn(
        'flex flex-wrap items-center gap-1.5 text-sm wrap-break-word text-muted-foreground sm:gap-2.5',
        className,
      )}
      {...props}
    />
  )
}

export const BreadcrumbItem = ({ className, ...props }: React.ComponentProps<'li'>) => {
  return <li className={cn('inline-flex items-center gap-1.5', className)} data-slot="breadcrumb-item" {...props} />
}

export const BreadcrumbLink = ({ className, render, ...props }: useRender.ComponentProps<'a'>) => {
  return useRender({
    defaultTagName: 'a',
    props: mergeProps<'a'>(
      {
        className: cn('transition-colors hover:text-foreground', className),
      },
      props,
    ),
    render,
    state: {
      slot: 'breadcrumb-link',
    },
  })
}

export const BreadcrumbPage = ({ className, ...props }: React.ComponentProps<'span'>) => {
  return (
    <span
      aria-current="page"
      aria-disabled="true"
      className={cn('font-normal text-foreground', className)}
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
      {children ?? <ChevronRightIcon />}
    </li>
  )
}

export const BreadcrumbEllipsis = ({ className, ...props }: React.ComponentProps<'span'>) => {
  return (
    <span
      aria-hidden="true"
      className={cn('flex size-5 items-center justify-center [&>svg]:size-4', className)}
      data-slot="breadcrumb-ellipsis"
      role="presentation"
      {...props}
    >
      <MoreHorizontalIcon />
      <span className="sr-only">More</span>
    </span>
  )
}
