import { InfoIcon } from 'lucide-react'
import type { PropsWithChildren, ReactNode } from 'react'
import { Label } from '../ui/label'
import { Tooltip, TooltipContent, TooltipTrigger } from '../ui/tooltip'

interface HintedLabelProps {
  hint?: ReactNode
}

export const HintedLabel = ({ children, hint }: PropsWithChildren<HintedLabelProps>) => {
  if (!hint) return <Label>{children}</Label>

  return (
    <div className="flex gap-2">
      <Label>{children}</Label>
      <Tooltip>
        <TooltipTrigger>
          <InfoIcon className="h-4 w-4" />
        </TooltipTrigger>
        <TooltipContent>{hint}</TooltipContent>
      </Tooltip>
    </div>
  )
}
