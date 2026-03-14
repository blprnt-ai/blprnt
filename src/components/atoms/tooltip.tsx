import {
  TooltipArrowPrimitive,
  TooltipContentPrimitive,
  type TooltipContentPrimitiveProps,
  TooltipPortalPrimitive,
  TooltipPrimitive,
  type TooltipPrimitiveProps,
  TooltipProviderPrimitive,
  type TooltipProviderPrimitiveProps,
  TooltipTriggerPrimitive,
  type TooltipTriggerPrimitiveProps,
} from '@/components/atoms/primitives/tooltip'
import { cn } from '@/lib/utils/cn'

type TooltipProviderProps = TooltipProviderPrimitiveProps

export const TooltipProvider = ({ delayDuration = 0, ...props }: TooltipProviderProps) => {
  return <TooltipProviderPrimitive delayDuration={delayDuration} {...props} />
}

export type TooltipProps = TooltipPrimitiveProps & {
  delayDuration?: TooltipPrimitiveProps['delayDuration']
}

export const Tooltip = ({ delayDuration = 0, ...props }: TooltipProps) => {
  return (
    <TooltipProvider delayDuration={delayDuration}>
      <TooltipPrimitive {...props} />
    </TooltipProvider>
  )
}

export type TooltipTriggerProps = TooltipTriggerPrimitiveProps

export const TooltipTrigger = ({ ...props }: TooltipTriggerProps) => {
  return <TooltipTriggerPrimitive {...props} />
}

export type TooltipContentProps = TooltipContentPrimitiveProps

export const TooltipContent = ({ className, sideOffset, children, ...props }: TooltipContentProps) => {
  return (
    <TooltipPortalPrimitive>
      <TooltipContentPrimitive
        sideOffset={sideOffset}
        className={cn(
          'bg-background border border-primary z-50 w-fit origin-(--radix-tooltip-content-transform-origin) rounded-md px-3 py-1.5 text-sm! text-balance',
          className,
        )}
        {...props}
      >
        {children}
        <TooltipArrowPrimitive className="bg-background fill-background border-primary border-b border-r z-50 size-2.5 -translate-y-1/2 rotate-45" />
      </TooltipContentPrimitive>
    </TooltipPortalPrimitive>
  )
}
