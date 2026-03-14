import { Info } from 'lucide-react'
import { Field } from '@/components/atoms/field'
import { InputGroup, InputGroupAddon, InputGroupInput } from '@/components/atoms/input-group'
import { Label } from '@/components/atoms/label'
import { TooltipMacro } from '@/components/atoms/tooltip-macro'
import { cn } from '@/lib/utils/cn'

interface SessionNameProps {
  className?: string
  sessionName: string
  onSetSessionName: (sessionName: string) => void
}

export const SessionName = ({ className, sessionName, onSetSessionName }: SessionNameProps) => {
  return (
    <Field className={cn(className)} data-tour="session-name">
      <InputGroup className="justify-between">
        <InputGroupAddon className="w-44 justify-start">
          <Label>Session Name</Label>
          <TooltipMacro tooltip="Give your session a name that will help you identify it in the future">
            <Info className="h-4 w-4 text-primary cursor-help" />
          </TooltipMacro>
        </InputGroupAddon>

        <InputGroupInput
          data-tour="session-name-input"
          id="session_name"
          value={sessionName}
          onChange={(e) => onSetSessionName(e.target.value)}
        />
      </InputGroup>
    </Field>
  )
}
