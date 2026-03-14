import { Tooltip, TooltipContent, TooltipTrigger } from './tooltip'

interface TooltipMacroProps {
  children: React.ReactNode
  tooltip: React.ReactNode
  withDelay?: boolean
  asChild?: boolean
  onClick?: (e: React.MouseEvent<HTMLButtonElement>) => void
  side?: 'top' | 'right' | 'bottom' | 'left'
  disabled?: boolean
}

export const TooltipMacro = ({
  children,
  disabled = false,
  tooltip,
  withDelay = false,
  side = 'top',
  onClick,
  asChild = true,
}: TooltipMacroProps) => {
  const delayDuration = withDelay ? 750 : 0

  if (disabled) return children

  const handleClick = (e: React.MouseEvent<HTMLButtonElement>) => {
    onClick?.(e)
  }

  return (
    <Tooltip delayDuration={delayDuration}>
      <TooltipTrigger asChild={asChild} autoFocus={false} tabIndex={-1} onClick={handleClick}>
        {children}
      </TooltipTrigger>
      <TooltipContent side={side}>{tooltip}</TooltipContent>
    </Tooltip>
  )
}
