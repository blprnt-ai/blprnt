import { AlertDialog as AlertDialogPrimitive } from '@base-ui/react/alert-dialog'
import type * as React from 'react'
import { buttonVariants } from '@/components/ui/button'
import { cn } from '@/lib/utils'

export const AlertDialog = ({ ...props }: AlertDialogPrimitive.Root.Props) => {
  return <AlertDialogPrimitive.Root data-slot="alert-dialog" {...props} />
}

export const AlertDialogTrigger = ({ ...props }: AlertDialogPrimitive.Trigger.Props) => {
  return <AlertDialogPrimitive.Trigger data-slot="alert-dialog-trigger" {...props} />
}

export const AlertDialogPortal = ({ ...props }: AlertDialogPrimitive.Portal.Props) => {
  return <AlertDialogPrimitive.Portal data-slot="alert-dialog-portal" {...props} />
}

export const AlertDialogOverlay = ({ className, ...props }: AlertDialogPrimitive.Backdrop.Props) => {
  return (
    <AlertDialogPrimitive.Backdrop
      className={cn(
        'fixed inset-0 z-50 bg-[color-mix(in_oklab,var(--primary)_12%,black)]/45 transition-opacity duration-150 data-ending-style:opacity-0 data-starting-style:opacity-0 supports-backdrop-filter:backdrop-blur-xs',
        className,
      )}
      data-slot="alert-dialog-overlay"
      {...props}
    />
  )
}

export const AlertDialogContent = ({ className, ...props }: AlertDialogPrimitive.Popup.Props) => {
  return (
    <AlertDialogPortal>
      <AlertDialogOverlay />
      <AlertDialogPrimitive.Popup
        className={cn(
          'fixed top-1/2 left-1/2 z-50 flex w-[calc(100vw-2rem)] max-w-md -translate-x-1/2 -translate-y-1/2 flex-col gap-5 rounded-sm bg-popover p-5 text-sm text-popover-foreground shadow-lg ring-1 ring-border/80 outline-hidden transition duration-150 data-ending-style:opacity-0 data-ending-style:scale-95 data-starting-style:opacity-0 data-starting-style:scale-95',
          className,
        )}
        data-slot="alert-dialog-content"
        {...props}
      />
    </AlertDialogPortal>
  )
}

export const AlertDialogHeader = ({ className, ...props }: React.ComponentProps<'div'>) => {
  return <div className={cn('flex flex-col gap-1.5', className)} data-slot="alert-dialog-header" {...props} />
}

export const AlertDialogFooter = ({ className, ...props }: React.ComponentProps<'div'>) => {
  return (
    <div
      className={cn('flex flex-col-reverse gap-2 sm:flex-row sm:justify-end', className)}
      data-slot="alert-dialog-footer"
      {...props}
    />
  )
}

export const AlertDialogTitle = ({ className, ...props }: AlertDialogPrimitive.Title.Props) => {
  return (
    <AlertDialogPrimitive.Title
      className={cn('font-heading text-base font-medium text-foreground', className)}
      data-slot="alert-dialog-title"
      {...props}
    />
  )
}

export const AlertDialogDescription = ({ className, ...props }: AlertDialogPrimitive.Description.Props) => {
  return (
    <AlertDialogPrimitive.Description
      className={cn('text-sm text-muted-foreground', className)}
      data-slot="alert-dialog-description"
      {...props}
    />
  )
}

export const AlertDialogCancel = ({ className, ...props }: AlertDialogPrimitive.Close.Props) => {
  return (
    <AlertDialogPrimitive.Close
      className={cn(buttonVariants({ variant: 'outline' }), className)}
      data-slot="alert-dialog-cancel"
      {...props}
    />
  )
}

export const AlertDialogAction = ({ className, ...props }: AlertDialogPrimitive.Close.Props) => {
  return (
    <AlertDialogPrimitive.Close
      className={cn(buttonVariants({ variant: 'destructive-outline' }), className)}
      data-slot="alert-dialog-action"
      {...props}
    />
  )
}
