import { Info } from 'lucide-react'
import { Field } from '@/components/atoms/field'
import { InputGroup, InputGroupAddon, InputGroupSwitch } from '@/components/atoms/input-group'
import { Label } from '@/components/atoms/label'
import { TooltipMacro } from '@/components/atoms/tooltip-macro'

interface OpenrouterWebSearchProps {
  webSearchEnabled: boolean
  onSetWebSearchEnabled: (webSearchEnabled: boolean) => void
}

export const WebSearchToggle = ({ webSearchEnabled, onSetWebSearchEnabled }: OpenrouterWebSearchProps) => {
  return (
    <Field>
      <InputGroup className="justify-between">
        <InputGroupAddon className="w-44 justify-start">
          <Label htmlFor="web_search_enabled">Enable web search</Label>
          <TooltipMacro tooltip="Allows model to use web search tool. Using this feature incurs additional costs.">
            <Info className="h-4 w-4 text-primary cursor-help" />
          </TooltipMacro>
        </InputGroupAddon>

        <InputGroupSwitch checked={webSearchEnabled} id="web_search_enabled" onCheckedChange={onSetWebSearchEnabled} />
      </InputGroup>
    </Field>
  )
}
