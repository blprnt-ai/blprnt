import { Close, Content, Description, Portal, Root, Title, Trigger } from '@radix-ui/react-dialog'
import { XIcon } from 'lucide-react'
import type * as React from 'react'
import { cn } from '@/lib/utils/cn'

export const Sheet = ({ ...props }: React.ComponentProps<typeof Root>) => {
  return <Root data-slot="sheet" modal={false} {...props} />
}

export const SheetTrigger = ({ ...props }: React.ComponentProps<typeof Trigger>) => {
  return <Trigger data-slot="sheet-trigger" {...props} />
}

export const SheetClose = ({ ...props }: React.ComponentProps<typeof Close>) => {
  return <Close data-slot="sheet-close" {...props} />
}

export const SheetPortal = ({ ...props }: React.ComponentProps<typeof Portal>) => {
  return <Portal data-slot="sheet-portal" {...props} />
}

export const SheetOverlay = () => {
  return (
    <div className="fixed inset-0 z-50">
      <div className="absolute inset-0 bg-black/10 backdrop-blur-xs" />
    </div>
  )
}

export const SheetContent = ({
  className,
  children,
  side = 'right',
  useOverlay = true,
  showCloseButton = true,
  ...props
}: React.ComponentProps<typeof Content> & {
  side?: 'top' | 'right' | 'bottom' | 'left'
  useOverlay?: boolean
  showCloseButton?: boolean
}) => {
  return (
    <SheetPortal>
      {useOverlay && <SheetOverlay />}
      <Content
        data-slot="sheet-content"
        className={cn(
          'bg-background data-[state=open]:animate-in data-[state=closed]:animate-out fixed z-50 flex flex-col shadow-lg transition ease-in-out data-[state=closed]:duration-200 data-[state=open]:duration-500 data-[state=closed]:delay-150',
          side === 'right' &&
            'data-[state=closed]:slide-out-to-right data-[state=open]:slide-in-from-right inset-y-0 right-0 h-full w-3/4 border-l sm:max-w-sm',
          side === 'left' &&
            'data-[state=closed]:slide-out-to-left data-[state=open]:slide-in-from-left inset-y-0 left-0 h-full w-3/4 border-r sm:max-w-sm',
          side === 'top' &&
            'data-[state=closed]:slide-out-to-top data-[state=open]:slide-in-from-top inset-x-0 top-0 h-auto border-b',
          side === 'bottom' &&
            'data-[state=closed]:slide-out-to-bottom data-[state=open]:slide-in-from-bottom inset-x-0 bottom-0 h-auto border-t',
          className,
        )}
        {...props}
      >
        {children}
        {showCloseButton && (
          <Close className="ring-offset-background data-[state=open]:bg-accent absolute top-4 right-4 rounded-xs opacity-70 transition-opacity hover:opacity-100 disabled:pointer-events-none">
            <XIcon className="size-4" />
            <span className="sr-only">Close</span>
          </Close>
        )}
      </Content>
    </SheetPortal>
  )
}

export const SheetHeader = ({ className, ...props }: React.ComponentProps<'div'>) => {
  return <div className={cn('flex flex-col gap-1.5 p-4', className)} data-slot="sheet-header" {...props} />
}

export const SheetFooter = ({ className, ...props }: React.ComponentProps<'div'>) => {
  return <div className={cn('mt-auto flex flex-col', className)} data-slot="sheet-footer" {...props} />
}

export const SheetFooterItem = ({ className, ...props }: React.ComponentProps<'div'>) => {
  return (
    <div
      data-slot="sheet-footer-item"
      className={cn(
        'flex items-center border-t border-border px-4 gap-2 text-sm h-8 bg-background hover:bg-background-2 transition-colors duration-300',
        className,
      )}
      {...props}
    />
  )
}

export const SheetTitle = ({ className, ...props }: React.ComponentProps<typeof Title>) => {
  return <Title className={cn('text-foreground font-medium', className)} data-slot="sheet-title" {...props} />
}

export const SheetDescription = ({ className, ...props }: React.ComponentProps<typeof Description>) => {
  return (
    <Description className={cn('text-muted-foreground text-sm', className)} data-slot="sheet-description" {...props} />
  )
}
