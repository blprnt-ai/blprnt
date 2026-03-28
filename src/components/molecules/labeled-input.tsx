import type { VariantProps } from 'class-variance-authority'
import type { ReactNode } from 'react'
import { Input, type inputVariants } from '@/components/ui/input'
import { cn } from '@/lib/utils'
import { HintedLabel } from './hinted-label'

interface LabeledInputProps {
  label: ReactNode
  value: string
  hint?: ReactNode
  placeholder?: string
  size?: VariantProps<typeof inputVariants>['size']
  inline?: boolean
  onChange: (value: string) => void
}

export const LabeledInput = ({
  label,
  value,
  hint,
  placeholder,
  size,
  inline = false,
  onChange,
}: LabeledInputProps) => {
  return (
    <div className={cn('flex flex-col gap-2', inline && 'flex-row items-center justify-between')}>
      <HintedLabel hint={hint}>{label}</HintedLabel>
      <Input
        required
        className={cn(inline && 'w-auto')}
        placeholder={placeholder}
        size={size}
        type="text"
        value={value}
        onChange={(e) => onChange(e.target.value)}
      />
    </div>
  )
}
