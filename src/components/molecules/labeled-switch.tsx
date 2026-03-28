import type { ReactNode } from 'react'
import { Switch } from '@/components/ui/switch'
import { cn } from '@/lib/utils'
import { HintedLabel } from './hinted-label'

interface LabeledSwitchProps {
  label: ReactNode
  value: boolean
  hint?: ReactNode
  inline?: boolean
  onChange: (value: boolean) => void
}

export const LabeledSwitch = ({ label, value, hint, inline = false, onChange }: LabeledSwitchProps) => {
  return (
    <div className={cn('flex flex-col gap-2', inline && 'flex-row items-center justify-between')}>
      <HintedLabel hint={hint}>{label}</HintedLabel>
      <Switch checked={value} onCheckedChange={onChange} />
    </div>
  )
}
