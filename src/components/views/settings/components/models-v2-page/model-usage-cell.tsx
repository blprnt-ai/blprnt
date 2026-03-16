import { useMemo } from 'react'
import { TooltipMacro } from '@/components/atoms/tooltip-macro'
import { getColorForValue, getContrastingTextColor, getLabelForValue } from './utils'

interface ModelUsageCellProps {
  minOutputPrice: number
  maxOutputPrice: number
  outputPrice: string
}

export const ModelUsageCell = ({ minOutputPrice, maxOutputPrice, outputPrice }: ModelUsageCellProps) => {
  const value = useMemo(() => parseFloat(outputPrice), [outputPrice])
  const color = useMemo(
    () => getColorForValue(minOutputPrice, maxOutputPrice, value),
    [minOutputPrice, maxOutputPrice, value],
  )
  const textColor = useMemo(() => getContrastingTextColor(color), [color])
  const label = useMemo(
    () => getLabelForValue(minOutputPrice, maxOutputPrice, value),
    [minOutputPrice, maxOutputPrice, value],
  )

  return (
    <TooltipMacro withDelay tooltip={`$${outputPrice}/M tokens`}>
      <div
        className="px-2 py-0.5 rounded-full w-fit text-xs"
        style={{
          background: color,
          color: textColor,
        }}
      >
        {label}
      </div>
    </TooltipMacro>
  )
}
