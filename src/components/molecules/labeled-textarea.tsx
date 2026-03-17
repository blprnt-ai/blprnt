import type { ReactNode } from 'react'
import { Textarea } from '@/components/ui/textarea'
import { HintedLabel } from './hinted-label'

interface LabeledTextareaProps {
  label: ReactNode
  value: string
  hint?: ReactNode
  placeholder?: string
  onChange: (value: string) => void
}

export const LabeledTextarea = ({ label, value, hint, placeholder, onChange }: LabeledTextareaProps) => {
  return (
    <div className="flex flex-col gap-2">
      <HintedLabel hint={hint}>{label}</HintedLabel>
      <Textarea placeholder={placeholder} value={value} onChange={(e) => onChange(e.target.value)} />
    </div>
  )
}
