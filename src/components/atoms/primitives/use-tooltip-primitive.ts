import { getStrictContext } from '@/lib/utils/get-strict-context'

export type TooltipContextType = {
  isOpen: boolean
  setIsOpen: (isOpen: boolean) => void
}

export const [TooltipProvider, useTooltip] = getStrictContext<TooltipContextType>('TooltipContext')
