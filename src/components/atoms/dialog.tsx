import { Close, Content, Description, Portal, Root, Title, Trigger } from '@radix-ui/react-dialog'
import { cva, type VariantProps } from 'class-variance-authority'
import { XIcon } from 'lucide-react'
import type React from 'react'
import { cn } from '@/lib/utils/cn'

export const Dialog = ({ ...props }: React.ComponentProps<typeof Root>) => {
  return <Root data-slot="dialog" {...props} />
}

export const DialogTrigger = ({ ...props }: React.ComponentProps<typeof Trigger>) => {
  return <Trigger data-slot="dialog-trigger" {...props} />
}

export const DialogPortal = ({ ...props }: React.ComponentProps<typeof Portal>) => {
  return <Portal data-slot="dialog-portal" {...props} />
}

export const DialogClose = ({ ...props }: React.ComponentProps<typeof Close>) => {
  return <Close data-slot="dialog-close" {...props} />
}

export const DialogOverlay = () => {
  return (
    <div className="fixed inset-0 z-50">
      <div className="absolute inset-0 bg-black/10 backdrop-blur-xs" />
    </div>
  )
}

const dialogVariants = cva(
  cn(
    'data-[state=open]:animate-in backdrop-blur-sm bg-gradient-glow-dark',
    'data-[state=open]:fade-in-0 data-[state=open]:zoom-in-95',
    'data-[state=closed]:zoom-out-95 data-[state=closed]:animate-out data-[state=closed]:fade-out-0',
    'fixed top-[50%] left-[50%] z-50 grid w-full gap-4',
    'max-w-[calc(100%-2rem)]',
    'translate-x-[-50%] translate-y-[-50%] duration-200',
    'rounded-lg border border-warn border-dashed p-6',
    'max-h-[calc(100vh-12rem)] overflow-y-auto',
  ),
  {
    defaultVariants: {
      size: 'md',
    },
    variants: {
      size: {
        '2xl': 'w-6xl',
        '3xl': 'w-7xl',
        lg: 'w-4xl',
        md: 'w-3xl',
        sm: 'w-2xl',
        xl: 'w-5xl',
        xs: 'w-xl',
      },
    },
  },
)

export const DialogContent = ({
  className,
  children,
  size,
  showCloseButton = true,
  ...props
}: React.ComponentProps<typeof Content> &
  VariantProps<typeof dialogVariants> & {
    showCloseButton?: boolean
  }) => {
  return (
    <DialogPortal data-slot="dialog-portal">
      <DialogOverlay />
      <Content className={cn(dialogVariants({ size }), className)} data-slot="dialog-content" {...props}>
        {children}
        {showCloseButton && (
          <Close
            className="ring-offset-background focus:ring-ring data-[state=open]:bg-primary data-[state=open]:text-muted-foreground absolute top-4 right-4 rounded-xs opacity-70 transition-opacity hover:opacity-100 focus:ring-2 focus:ring-offset-2 focus:outline-hidden disabled:pointer-events-none [&_svg]:pointer-events-none [&_svg]:shrink-0 [&_svg:not([class*='size-'])]:size-4 cursor-pointer max-h-screen"
            data-slot="dialog-close"
            tabIndex={-1}
          >
            <XIcon />
            <span className="sr-only">Close</span>
          </Close>
        )}
      </Content>
    </DialogPortal>
  )
}

export const DialogHeader = ({ className, ...props }: React.ComponentProps<'div'>) => {
  return (
    <div
      className={cn('flex flex-col gap-2 text-center sm:text-left', className)}
      data-slot="dialog-header"
      {...props}
    />
  )
}

export const DialogFooter = ({ className, ...props }: React.ComponentProps<'div'>) => {
  return (
    <div
      className={cn('flex flex-col-reverse gap-2 sm:flex-row sm:justify-end', className)}
      data-slot="dialog-footer"
      {...props}
    />
  )
}

export const DialogTitle = ({ className, ...props }: React.ComponentProps<typeof Title>) => {
  return (
    <Title className={cn('text-lg leading-none font-medium! m-0!', className)} data-slot="dialog-title" {...props} />
  )
}

export const DialogDescription = ({ className, ...props }: React.ComponentProps<typeof Description>) => {
  return (
    <Description className={cn('text-muted-foreground text-sm', className)} data-slot="dialog-description" {...props} />
  )
}
