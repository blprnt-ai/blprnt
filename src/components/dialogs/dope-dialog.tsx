import { Close, Content, Description, Portal, Root, Title, Trigger } from '@radix-ui/react-dialog'
import { type PropsWithChildren, useState } from 'react'
import { Button } from '@/components/atoms/button'
import { cn } from '@/lib/utils/cn'

export const DopeDialog = ({ ...props }: React.ComponentProps<typeof Root>) => {
  return <Root data-slot="dialog" {...props} />
}

export const DopeDialogTrigger = ({ ...props }: React.ComponentProps<typeof Trigger>) => {
  return <Trigger data-slot="dialog-trigger" {...props} />
}

export const DopeDialogPortal = ({ ...props }: React.ComponentProps<typeof Portal>) => {
  return <Portal data-slot="dialog-portal" {...props} />
}

export const DopeDialogClose = ({ ...props }: React.ComponentProps<typeof Close>) => {
  return <Close data-slot="dialog-close" {...props} />
}

export const DopeDialogContent = ({ className, ...props }: React.ComponentProps<typeof Content>) => {
  return (
    <Content
      data-slot="dialog-content"
      className={cn(
        'bg-gradient-glow',
        'data-[state=open]:animate-in data-[state=open]:fade-in-0 data-[state=open]:zoom-in-15 data-[state=open]:slide-in-from-bottom-50',
        'data-[state=closed]:zoom-out-15 data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=closed]:slide-out-to-top-50',
        'fixed top-[50%] left-[50%] z-50 grid w-full gap-4',
        'max-w-[calc(100%-2rem)]',
        'translate-x-[-50%] translate-y-[-50%] duration-200',
        'rounded-lg border border-warn border-dashed p-6',
        'max-h-[calc(100vh-12rem)] overflow-y-auto',
        className,
      )}
      {...props}
    />
  )
}

export const DopeDialogTitle = ({ ...props }: React.ComponentProps<typeof Title>) => {
  return <Title data-slot="dialog-title" {...props} />
}

export const DopeDialogDescription = ({ ...props }: React.ComponentProps<typeof Description>) => {
  return <Description data-slot="dialog-description" {...props} />
}

export const DopeDialogOverlay = () => {
  return (
    <div className="fixed inset-0 z-50">
      <div className="absolute inset-0 bg-slate-800/40 backdrop-blur" />
    </div>
  )
}

interface ConfirmProps extends PropsWithChildren {
  cancelLabel: string
  okLabel: string
  title: string
  body?: React.ReactNode
  className?: string
  onCancel: () => void
  onOk: () => void
}

export const Confirm = ({ children, cancelLabel, okLabel, title, body, className, onCancel, onOk }: ConfirmProps) => {
  const [isOpen, setIsOpen] = useState(false)

  const handleOpenChange = (isOpen: boolean) => {
    setIsOpen(isOpen)
  }

  const handleCancel = () => {
    setIsOpen(false)
    onCancel()
  }

  const handleOk = () => {
    setIsOpen(false)
    onOk()
  }
  return (
    <DopeDialog open={isOpen} onOpenChange={handleOpenChange}>
      <DopeDialogTrigger asChild onClick={() => handleOpenChange(true)}>
        {children}
      </DopeDialogTrigger>
      <DopeDialogPortal>
        <DopeDialogOverlay />
        <DopeDialogContent className={cn('max-w-xs', className)}>
          <DopeDialogTitle className="m-0! text-center">{title}</DopeDialogTitle>
          {body && <DopeDialogDescription>{body}</DopeDialogDescription>}
          <div className="flex justify-center gap-8">
            <Button className="rounded-xl" variant="outline-ghost" onClick={handleCancel}>
              {cancelLabel}
            </Button>
            <Button className="rounded-xl" variant="outline-ghost" onClick={handleOk}>
              {okLabel}
            </Button>
          </div>
        </DopeDialogContent>
      </DopeDialogPortal>
    </DopeDialog>
  )
}
