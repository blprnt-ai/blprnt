import { InfoIcon } from 'lucide-react'
import { Input } from '@/components/atoms/input'
import { TooltipMacro } from '@/components/atoms/tooltip-macro'

interface FormFieldProps {
  label: string
  value: string
  helpText?: string
  onChange: (value: string) => void
  type?: 'text' | 'number'
}

export const FormField = ({ label, helpText, value, onChange, type = 'text' }: FormFieldProps) => (
  <label className="flex flex-col gap-1.5">
    <span className="text-sm text-muted-foreground flex items-center gap-1">
      {label}
      {helpText && (
        <TooltipMacro tooltip={helpText}>
          <InfoIcon className="h-3.5 w-3.5" />
        </TooltipMacro>
      )}
    </span>
    <Input type={type} value={value} onChange={(event) => onChange(event.target.value)} />
  </label>
)
