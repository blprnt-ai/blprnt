import type { ReactNode } from 'react'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { cn } from '@/lib/utils'
import { HintedLabel } from './hinted-label'

export interface LabeledSelectOption {
  label: ReactNode
  value: string
}

interface LabeledSelectProps<T extends string> {
  label: ReactNode
  hint?: ReactNode
  value: T
  placeholder?: string
  selectedValue: ReactNode
  onChange: (value: T | null) => void
  options: LabeledSelectOption[]
  fullWidth?: boolean
}

export const LabeledSelect = <T extends string>({
  label,
  hint,
  value,
  placeholder,
  selectedValue,
  onChange,
  options,
  fullWidth = true,
}: LabeledSelectProps<T>) => {
  return (
    <div className="flex flex-col gap-2">
      <HintedLabel hint={hint}>{label}</HintedLabel>

      <Select value={value} onValueChange={onChange}>
        <SelectTrigger className={cn(fullWidth && 'w-full')}>
          <SelectValue placeholder={placeholder}>{selectedValue}</SelectValue>
        </SelectTrigger>
        <SelectContent>
          {options.map((option) => (
            <SelectItem key={option.value} value={option.value}>
              {option.label}
            </SelectItem>
          ))}
        </SelectContent>
      </Select>
    </div>
  )
}
