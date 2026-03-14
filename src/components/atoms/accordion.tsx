import { ChevronDownIcon } from 'lucide-react'

import {
  AccordionContentPrimitive,
  type AccordionContentPrimitiveProps,
  AccordionHeaderPrimitive,
  AccordionItemPrimitive,
  type AccordionItemPrimitiveProps,
  AccordionPrimitive,
  type AccordionPrimitiveProps,
  AccordionTriggerPrimitive,
  type AccordionTriggerPrimitiveProps,
} from '@/components/atoms/primitives/accordion'
import { cn } from '@/lib/utils/cn'

export type AccordionProps = AccordionPrimitiveProps

export const Accordion = (props: AccordionProps) => {
  return <AccordionPrimitive {...props} />
}

export type AccordionItemProps = AccordionItemPrimitiveProps

export const AccordionItem = ({ className, ...props }: AccordionItemProps) => {
  return <AccordionItemPrimitive className={cn('border-b last:border-b-0', className)} {...props} />
}

export type AccordionTriggerProps = AccordionTriggerPrimitiveProps & {
  showArrow?: boolean
}

export const AccordionTrigger = ({ className, children, showArrow = true, ...props }: AccordionTriggerProps) => {
  return (
    <AccordionHeaderPrimitive className="flex">
      <AccordionTriggerPrimitive
        className={cn(
          'focus-visible:border-ring focus-visible:ring-ring/50 flex flex-1 items-start justify-between gap-4 rounded-md py-4 text-left text-sm font-medium transition-all outline-none hover:underline focus-visible:ring-[3px] disabled:pointer-events-none disabled:opacity-50 [&[data-state=open]>svg]:rotate-180',
          className,
        )}
        {...props}
      >
        {children}
        {showArrow && (
          <ChevronDownIcon className="text-muted-foreground pointer-events-none size-4 shrink-0 translate-y-0.5 transition-transform duration-200" />
        )}
      </AccordionTriggerPrimitive>
    </AccordionHeaderPrimitive>
  )
}

export type AccordionContentProps = AccordionContentPrimitiveProps

export const AccordionContent = ({ className, children, ...props }: AccordionContentProps) => {
  return (
    <AccordionContentPrimitive {...props}>
      <div className={cn('text-sm pt-0 pb-4', className)}>{children}</div>
    </AccordionContentPrimitive>
  )
}
