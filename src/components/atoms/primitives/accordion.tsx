import { AnimatePresence, type HTMLMotionProps, motion } from 'motion/react'
import { Accordion } from 'radix-ui'
import React from 'react'
import { useControlledState } from '@/hooks/use-controlled-state'
import {
  AccordionItemProviderPrimitive,
  AccordionProviderPrimitive,
  useAccordionItemPrimitive,
  useAccordionPrimitive,
} from './use-accordian-primitive'

export type AccordionPrimitiveProps = React.ComponentProps<typeof Accordion.Root>

export const AccordionPrimitive = (props: AccordionPrimitiveProps) => {
  const [value, setValue] = useControlledState<string | string[] | undefined>({
    defaultValue: props?.defaultValue,
    onChange: props?.onValueChange as (value: string | string[] | undefined) => void,
    value: props?.value,
  })

  return (
    <AccordionProviderPrimitive value={{ setValue, value }}>
      <Accordion.Root data-slot="accordion" {...props} onValueChange={setValue} />
    </AccordionProviderPrimitive>
  )
}

export type AccordionItemPrimitiveProps = React.ComponentProps<typeof Accordion.Item>

export const AccordionItemPrimitive = (props: AccordionItemPrimitiveProps) => {
  const { value } = useAccordionPrimitive()
  const [isOpen, setIsOpen] = React.useState(value?.includes(props?.value) ?? false)

  React.useEffect(() => {
    setIsOpen(value?.includes(props?.value) ?? false)
  }, [value, props?.value])

  return (
    <AccordionItemProviderPrimitive value={{ isOpen, setIsOpen, value: props.value }}>
      <Accordion.Item data-slot="accordion-item" {...props} />
    </AccordionItemProviderPrimitive>
  )
}

export type AccordionHeaderPrimitiveProps = React.ComponentProps<typeof Accordion.Header>

export const AccordionHeaderPrimitive = (props: AccordionHeaderPrimitiveProps) => {
  return <Accordion.Header data-slot="accordion-header" {...props} />
}

export type AccordionTriggerPrimitiveProps = React.ComponentProps<typeof Accordion.Trigger>

export const AccordionTriggerPrimitive = (props: AccordionTriggerPrimitiveProps) => {
  return <Accordion.Trigger data-slot="accordion-trigger" {...props} />
}

export type AccordionContentPrimitiveProps = Omit<
  React.ComponentProps<typeof Accordion.Content>,
  'asChild' | 'forceMount'
> &
  HTMLMotionProps<'div'> & {
    keepRendered?: boolean
  }

export const AccordionContentPrimitive = ({
  keepRendered = false,
  transition = { duration: 0.35, ease: 'easeInOut' },
  ...props
}: AccordionContentPrimitiveProps) => {
  const { isOpen } = useAccordionItemPrimitive()

  return (
    <AnimatePresence>
      {keepRendered ? (
        <Accordion.Content asChild forceMount>
          <motion.div
            key="accordion-content"
            data-slot="accordion-content"
            initial={{ '--mask-stop': '0%', height: 0, opacity: 0, y: 20 }}
            transition={transition}
            animate={
              isOpen
                ? { '--mask-stop': '100%', height: 'auto', opacity: 1, y: 0 }
                : { '--mask-stop': '0%', height: 0, opacity: 0, y: 20 }
            }
            style={{
              maskImage: 'linear-gradient(black var(--mask-stop), transparent var(--mask-stop))',
              overflow: 'hidden',
              WebkitMaskImage: 'linear-gradient(black var(--mask-stop), transparent var(--mask-stop))',
            }}
            {...props}
          />
        </Accordion.Content>
      ) : (
        isOpen && (
          <Accordion.Content asChild forceMount>
            <motion.div
              key="accordion-content"
              data-slot="accordion-content"
              exit={{ '--mask-stop': '0%', height: 0, opacity: 0, y: 20 }}
              initial={{ '--mask-stop': '0%', height: 0, opacity: 0, y: 20 }}
              transition={transition}
              animate={{
                '--mask-stop': '100%',
                height: 'auto',
                opacity: 1,
                y: 0,
              }}
              style={{
                maskImage: 'linear-gradient(black var(--mask-stop), transparent var(--mask-stop))',
                overflow: 'hidden',
                WebkitMaskImage: 'linear-gradient(black var(--mask-stop), transparent var(--mask-stop))',
              }}
              {...props}
            />
          </Accordion.Content>
        )
      )}
    </AnimatePresence>
  )
}
