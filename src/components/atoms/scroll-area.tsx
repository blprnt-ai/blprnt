import { Corner, Root, ScrollAreaScrollbar, ScrollAreaThumb, Viewport } from '@radix-ui/react-scroll-area'
import type React from 'react'
import { cn } from '@/lib/utils/cn'

export const ScrollArea = ({ className, children, ...props }: React.ComponentProps<typeof Root>) => {
  return (
    <Root className={cn('relative', className)} data-slot="scroll-area" {...props}>
      <Viewport
        className="focus-visible:ring-ring/50 size-full rounded-[inherit] transition-[color,box-shadow] outline-none focus-visible:ring-[3px] focus-visible:outline-1"
        data-slot="scroll-area-viewport"
      >
        {children}
      </Viewport>
      <ScrollBar />
      <Corner />
    </Root>
  )
}

export const ScrollBar = ({
  className,
  orientation = 'vertical',
  ...props
}: React.ComponentProps<typeof ScrollAreaScrollbar>) => {
  return (
    <ScrollAreaScrollbar
      data-slot="scroll-area-scrollbar"
      orientation={orientation}
      className={cn(
        'flex touch-none p-px transition-colors select-none',
        orientation === 'vertical' && 'h-full w-2.5 border-l border-l-transparent',
        orientation === 'horizontal' && 'h-2.5 flex-col border-t border-t-transparent',
        className,
      )}
      {...props}
    >
      <ScrollAreaThumb className="bg-border relative flex-1 rounded-full" data-slot="scroll-area-thumb" />
    </ScrollAreaScrollbar>
  )
}
