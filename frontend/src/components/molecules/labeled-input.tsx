import type { VariantProps } from 'class-variance-authority'
import type { ReactNode } from 'react'
import { Input, type inputVariants } from '@/components/ui/input'
import { cn } from '@/lib/utils'
import { HintedLabel } from './hinted-label'

interface LabeledInputProps {
  label: ReactNode
  value: string
  hint?: ReactNode
  className?: string
  placeholder?: string
  type?: React.ComponentProps<'input'>['type']
  autoComplete?: string
  size?: VariantProps<typeof inputVariants>['size']
  inline?: boolean
  onChange: (value: string) => void
}

export const LabeledInput = ({
  label,
  value,
  hint,
  className,
  placeholder,
  type = 'text',
  autoComplete,
  size,
  inline = false,
  onChange,
}: LabeledInputProps) => {
  return (
    <div className={cn('flex flex-col gap-2', inline && 'flex-row items-center justify-between', className)}>
      <HintedLabel hint={hint}>{label}</HintedLabel>
      <Input
        required
        autoComplete={autoComplete}
        className={cn(inline && 'w-auto')}
        placeholder={placeholder}
        size={size}
        type={type}
        value={value}
        onChange={(e) => onChange(e.target.value)}
      />
    </div>
  )
}
