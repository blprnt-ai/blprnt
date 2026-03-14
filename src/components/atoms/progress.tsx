import { Indicator, Root } from '@radix-ui/react-progress'
import type React from 'react'

import { cn } from '@/lib/utils/cn'

export function Progress({ className, value, ...props }: React.ComponentProps<typeof Root>) {
  return (
    <Root
      className={cn('bg-primary/20 relative h-2 w-full overflow-hidden rounded-full', className)}
      data-slot="progress"
      {...props}
    >
      <Indicator
        className="bg-primary h-full w-full flex-1 transition-all"
        data-slot="progress-indicator"
        style={{ transform: `translateX(-${100 - (value || 0)}%)` }}
      />
    </Root>
  )
}
