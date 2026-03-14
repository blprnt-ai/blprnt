import { Root, Thumb } from '@radix-ui/react-switch'
import type React from 'react'

import { cn } from '@/lib/utils/cn'

export const Switch = ({ className, ...props }: React.ComponentProps<typeof Root>) => {
  return (
    <Root
      data-slot="switch"
      className={cn(
        'peer data-[state=checked]:bg-cyan-600 data-[state=unchecked]:bg-gray-700 focus-visible:border-cyan-500 focus-visible:ring-cyan-500/50 inline-flex h-[1.15rem] w-8 shrink-0 items-center rounded-full border border-transparent shadow-xs transition-all outline-none focus-visible:ring-[3px] disabled:cursor-not-allowed disabled:opacity-50',
        className,
      )}
      {...props}
    >
      <Thumb
        data-slot="switch-thumb"
        className={cn(
          'bg-white pointer-events-none block size-4 rounded-full ring-0 transition-transform data-[state=checked]:translate-x-[calc(100%-2px)] data-[state=unchecked]:translate-x-0',
        )}
      />
    </Root>
  )
}

export const SwitchSmall = ({ className, ...props }: React.ComponentProps<typeof Root>) => {
  return (
    <Root
      data-slot="switch"
      className={cn(
        'h-3 w-6 shrink-0',
        'peer data-[state=checked]:bg-cyan-600 data-[state=unchecked]:bg-gray-700 focus-visible:border-cyan-500 focus-visible:ring-cyan-500/50 inline-flex',
        'items-center rounded-full border border-transparent shadow-xs transition-all outline-none focus-visible:ring-[3px] disabled:cursor-not-allowed disabled:opacity-50',
        className,
      )}
      {...props}
    >
      <Thumb
        data-slot="switch-thumb"
        className={cn(
          'bg-white pointer-events-none block size-3 rounded-full ring-0 transition-transform data-[state=checked]:translate-x-[calc(100%-2px)] data-[state=unchecked]:translate-x-0',
        )}
      />
    </Root>
  )
}
