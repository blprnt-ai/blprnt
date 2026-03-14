import { AnimatePresence, type HTMLMotionProps, motion } from 'motion/react'
import { Tooltip } from 'radix-ui'
import type React from 'react'
import { useControlledState } from '@/hooks/use-controlled-state'
import { TooltipProvider, useTooltip } from './use-tooltip-primitive'

export type TooltipProviderPrimitiveProps = React.ComponentProps<typeof Tooltip.Provider>

export const TooltipProviderPrimitive = (props: TooltipProviderPrimitiveProps) => {
  return <Tooltip.Provider data-slot="tooltip-provider" {...props} />
}

export type TooltipPrimitiveProps = React.ComponentProps<typeof Tooltip.Root>

export const TooltipPrimitive = (props: TooltipPrimitiveProps) => {
  const [isOpen, setIsOpen] = useControlledState({
    defaultValue: props?.defaultOpen,
    onChange: props?.onOpenChange,
    value: props?.open,
  })

  return (
    <TooltipProvider value={{ isOpen, setIsOpen }}>
      <Tooltip.Root data-slot="tooltip" {...props} onOpenChange={setIsOpen} />
    </TooltipProvider>
  )
}

export type TooltipTriggerPrimitiveProps = React.ComponentProps<typeof Tooltip.Trigger>

export const TooltipTriggerPrimitive = (props: TooltipTriggerPrimitiveProps) => {
  return <Tooltip.Trigger data-slot="tooltip-trigger" {...props} />
}

export type TooltipPortalPrimitiveProps = Omit<React.ComponentProps<typeof Tooltip.Portal>, 'forceMount'>

export const TooltipPortalPrimitive = (props: TooltipPortalPrimitiveProps) => {
  const { isOpen } = useTooltip()

  return (
    <AnimatePresence>{isOpen && <Tooltip.Portal forceMount data-slot="tooltip-portal" {...props} />}</AnimatePresence>
  )
}

export type TooltipContentPrimitiveProps = Omit<
  React.ComponentProps<typeof Tooltip.Content>,
  'forceMount' | 'asChild'
> &
  HTMLMotionProps<'div'>

export const TooltipContentPrimitive = ({
  onEscapeKeyDown,
  onPointerDownOutside,
  side,
  sideOffset,
  align,
  alignOffset,
  avoidCollisions,
  collisionBoundary,
  collisionPadding,
  arrowPadding,
  sticky,
  hideWhenDetached,
  transition = { damping: 25, stiffness: 300, type: 'spring' },
  ...props
}: TooltipContentPrimitiveProps) => {
  return (
    <Tooltip.Content
      asChild
      forceMount
      align={align}
      alignOffset={alignOffset}
      arrowPadding={arrowPadding}
      avoidCollisions={avoidCollisions}
      collisionBoundary={collisionBoundary}
      collisionPadding={collisionPadding}
      hideWhenDetached={hideWhenDetached}
      side={side}
      sideOffset={sideOffset}
      sticky={sticky}
      onEscapeKeyDown={onEscapeKeyDown}
      onPointerDownOutside={onPointerDownOutside}
    >
      <motion.div
        key="popover-content"
        animate={{ opacity: 1, scale: 1 }}
        data-slot="popover-content"
        exit={{ opacity: 0, scale: 0.5 }}
        initial={{ opacity: 0, scale: 0.5 }}
        transition={transition}
        {...props}
      />
    </Tooltip.Content>
  )
}

export type TooltipArrowPrimitiveProps = React.ComponentProps<typeof Tooltip.Arrow>

export const TooltipArrowPrimitive = (props: TooltipArrowPrimitiveProps) => {
  return <Tooltip.Arrow data-slot="tooltip-arrow" {...props} />
}
