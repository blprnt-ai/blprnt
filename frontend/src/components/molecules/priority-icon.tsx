import { AlertTriangle, ArrowDown, ArrowUp, Minus } from 'lucide-react'
import { useState } from 'react'
import { Button } from '@/components/ui/button'
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover'
import { priorityColor, priorityColorDefault } from '@/lib/status-colors'
import { cn } from '@/lib/utils'

const priorityConfig: Record<string, { icon: typeof ArrowUp; color: string; label: string }> = {
  critical: { color: priorityColor.critical ?? priorityColorDefault, icon: AlertTriangle, label: 'Critical' },
  high: { color: priorityColor.high ?? priorityColorDefault, icon: ArrowUp, label: 'High' },
  low: { color: priorityColor.low ?? priorityColorDefault, icon: ArrowDown, label: 'Low' },
  medium: { color: priorityColor.medium ?? priorityColorDefault, icon: Minus, label: 'Medium' },
}

const allPriorities = ['critical', 'high', 'medium', 'low']

interface PriorityIconProps {
  priority: string
  onChange?: (priority: string) => void
  className?: string
  showLabel?: boolean
}

export function PriorityIcon({ priority, onChange, className, showLabel }: PriorityIconProps) {
  const [open, setOpen] = useState(false)
  const config = priorityConfig[priority] ?? priorityConfig.medium!
  const Icon = config.icon

  const icon = (
    <span
      className={cn(
        'inline-flex items-center justify-center shrink-0',
        config.color,
        onChange && !showLabel && 'cursor-pointer',
        className,
      )}
    >
      <Icon className="h-3.5 w-3.5" />
    </span>
  )

  if (!onChange)
    return showLabel ? (
      <span className="inline-flex items-center gap-1.5">
        {icon}
        <span className="text-sm">{config.label}</span>
      </span>
    ) : (
      icon
    )

  const trigger = showLabel ? (
    <button className="inline-flex items-center gap-1.5 cursor-pointer hover:bg-accent/50 rounded px-1 -mx-1 py-0.5 transition-colors">
      {icon}
      <span className="text-sm">{config.label}</span>
    </button>
  ) : (
    icon
  )

  return (
    <Popover open={open} onOpenChange={setOpen}>
      <PopoverTrigger>{trigger}</PopoverTrigger>
      <PopoverContent align="start" className="w-36 p-1">
        {allPriorities.map((p) => {
          const c = priorityConfig[p]!
          const PIcon = c.icon
          return (
            <Button
              key={p}
              className={cn('w-full justify-start gap-2 text-xs', p === priority && 'bg-accent')}
              size="sm"
              variant="ghost"
              onClick={() => {
                onChange(p)
                setOpen(false)
              }}
            >
              <PIcon className={cn('h-3.5 w-3.5', c.color)} />
              {c.label}
            </Button>
          )
        })}
      </PopoverContent>
    </Popover>
  )
}
