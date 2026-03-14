import { Label } from '@/components/atoms/label'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/atoms/select'
import { USAGE_LABELS, type UsageLabel } from './utils'

export type UsageFilterValue = UsageLabel | 'all'

interface UsageFilterSelectProps {
  value: UsageFilterValue
  onChange: (value: UsageFilterValue) => void
}

export const UsageFilterSelect = ({ value, onChange }: UsageFilterSelectProps) => {
  return (
    <div className="flex flex-col gap-1.5">
      <Label className="text-xs text-muted-foreground">Usage</Label>
      <Select value={value} onValueChange={(v) => onChange(v as UsageFilterValue)}>
        <SelectTrigger className="w-32">
          <SelectValue />
        </SelectTrigger>
        <SelectContent>
          <SelectItem value="all">All</SelectItem>
          {USAGE_LABELS.map((label) => (
            <SelectItem key={label} value={label}>
              {label}
            </SelectItem>
          ))}
        </SelectContent>
      </Select>
    </div>
  )
}
