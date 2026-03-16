import { Checkbox } from '@/components/atoms/checkbox'
import { Label } from '@/components/atoms/label'

interface ReasoningFilterProps {
  checked: boolean
  onChange: (checked: boolean) => void
}

export const ReasoningFilter = ({ checked, onChange }: ReasoningFilterProps) => {
  return (
    <div className="flex items-center gap-2 h-9">
      <Checkbox
        checked={checked}
        id="reasoning-filter"
        onCheckedChange={(checkedValue) => onChange(checkedValue === true)}
      />
      <Label className="text-sm cursor-pointer" htmlFor="reasoning-filter">
        Reasoning only
      </Label>
    </div>
  )
}
