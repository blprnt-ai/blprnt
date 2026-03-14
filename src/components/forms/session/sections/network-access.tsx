import { Info } from 'lucide-react'
import { Field } from '@/components/atoms/field'
import { InputGroup, InputGroupAddon, InputGroupSwitch } from '@/components/atoms/input-group'
import { Label } from '@/components/atoms/label'
import { TooltipMacro } from '@/components/atoms/tooltip-macro'
import { cn } from '@/lib/utils/cn'

interface NetworkAccessProps {
  className?: string
  isYolo: boolean
  networkAccess: boolean
  onSetNetworkAccess: (networkAccess: boolean) => void
}

export const NetworkAccess = ({ className, isYolo, networkAccess, onSetNetworkAccess }: NetworkAccessProps) => {
  return (
    <div
      className={cn(
        'transition-all duration-700 overflow-hidden',
        isYolo ? 'max-h-0 opacity-0' : 'max-h-20 opacity-100',
        className,
      )}
    >
      <Field>
        <InputGroup className="justify-between">
          <InputGroupAddon className="w-44 justify-start">
            <Label htmlFor="network_access">Network Access</Label>

            <TooltipMacro tooltip="Allow network requests and external API calls">
              <Info className="h-4 w-4 text-primary cursor-help" />
            </TooltipMacro>
          </InputGroupAddon>

          <InputGroupSwitch checked={networkAccess} id="network_access" onCheckedChange={onSetNetworkAccess} />
        </InputGroup>
      </Field>
    </div>
  )
}
