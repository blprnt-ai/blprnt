import { Input } from '@/components/atoms/input'

interface FormFieldProps {
  label: string
  value: string
  onChange: (value: string) => void
  type?: 'text' | 'number'
}

export const FormField = ({ label, value, onChange, type = 'text' }: FormFieldProps) => (
  <label className="flex flex-col gap-1.5">
    <span className="text-xs text-muted-foreground">{label}</span>
    <Input type={type} value={value} onChange={(event) => onChange(event.target.value)} />
  </label>
)