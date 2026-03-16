import { Label } from '@/components/atoms/label'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/atoms/select'

export type EnabledFilterValue = 'all' | 'enabled' | 'disabled'

interface StatusFilterProps {
  value: EnabledFilterValue
  onChange: (value: EnabledFilterValue) => void
}

export const StatusFilter = ({ value, onChange }: StatusFilterProps) => {
  return (
    <div className="flex flex-col gap-1.5">
      <Label className="text-xs text-muted-foreground">Status</Label>
      <Select value={value} onValueChange={(v) => onChange(v as EnabledFilterValue)}>
        <SelectTrigger className="w-32">
          <SelectValue />
        </SelectTrigger>
        <SelectContent>
          <SelectItem value="all">All</SelectItem>
          <SelectItem value="enabled">Enabled</SelectItem>
          <SelectItem value="disabled">Disabled</SelectItem>
        </SelectContent>
      </Select>
    </div>
  )
}
