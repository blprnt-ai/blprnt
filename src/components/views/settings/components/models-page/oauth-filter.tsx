import { Checkbox } from '@/components/atoms/checkbox'
import { Label } from '@/components/atoms/label'

interface OauthFilterProps {
  checked: boolean
  onChange: (checked: boolean) => void
}

export const OauthFilter = ({ checked, onChange }: OauthFilterProps) => {
  return (
    <div className="flex items-center gap-2 h-9">
      <Checkbox
        checked={checked}
        id="oauth-filter"
        onCheckedChange={(checkedValue) => onChange(checkedValue === true)}
      />
      <Label className="text-sm cursor-pointer" htmlFor="oauth-filter">
        API Key only
      </Label>
    </div>
  )
}
