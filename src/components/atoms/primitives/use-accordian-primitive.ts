import { getStrictContext } from '@/lib/utils/get-strict-context'

export type AccordionContextPrimitiveType = {
  value: string | string[] | undefined
  setValue: (value: string | string[] | undefined) => void
}

export const [AccordionProviderPrimitive, useAccordionPrimitive] =
  getStrictContext<AccordionContextPrimitiveType>('AccordionContext')

export type AccordionItemContextPrimitiveType = {
  value: string
  isOpen: boolean
  setIsOpen: (open: boolean) => void
}

export const [AccordionItemProviderPrimitive, useAccordionItemPrimitive] =
  getStrictContext<AccordionItemContextPrimitiveType>('AccordionItemContext')
