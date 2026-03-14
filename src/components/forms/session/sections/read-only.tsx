import { Info } from 'lucide-react'
import { Field } from '@/components/atoms/field'
import { InputGroup, InputGroupAddon, InputGroupSwitch } from '@/components/atoms/input-group'
import { Label } from '@/components/atoms/label'
import { TooltipMacro } from '@/components/atoms/tooltip-macro'
import { cn } from '@/lib/utils/cn'

interface ReadOnlyProps {
  className?: string
  readOnly: boolean
  yolo: boolean

  onSetReadOnly: (readOnly: boolean) => void
  onSetYolo: (yolo: boolean) => void
}

export const ReadOnly = ({ readOnly, yolo, onSetReadOnly, onSetYolo }: ReadOnlyProps) => {
  const handleReadOnlyChange = (checked: boolean) => {
    onSetReadOnly(checked)
    if (checked) onSetYolo(false)
  }

  return (
    <div
      className={cn(
        'transition-all duration-700 overflow-hidden',
        yolo ? 'max-h-0 opacity-0' : 'max-h-20 opacity-100 mb-4',
      )}
    >
      <Field>
        <InputGroup className="justify-between">
          <InputGroupAddon className="w-44 justify-start">
            <Label htmlFor="read_only">Read Only</Label>
            <TooltipMacro tooltip="Prevent all file modifications and writes">
              <Info className="h-4 w-4 text-primary cursor-help" />
            </TooltipMacro>
          </InputGroupAddon>

          <InputGroupSwitch checked={readOnly} id="read_only" onCheckedChange={handleReadOnlyChange} />
        </InputGroup>
      </Field>
    </div>
  )
}
