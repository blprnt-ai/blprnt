import { Checkbox } from '@/components/atoms/checkbox'
import { Label } from '@/components/atoms/label'

interface AutoRouterFilterProps {
  checked: boolean
  onChange: (checked: boolean) => void
}

export const AutoRouterFilter = ({ checked, onChange }: AutoRouterFilterProps) => {
  return (
    <div className="flex items-center gap-2 h-9">
      <Checkbox
        checked={checked}
        id="auto-router-filter"
        onCheckedChange={(checkedValue) => onChange(checkedValue === true)}
      />
      <Label className="text-sm cursor-pointer" htmlFor="auto-router-filter">
        Auto Router only
      </Label>
    </div>
  )
}
