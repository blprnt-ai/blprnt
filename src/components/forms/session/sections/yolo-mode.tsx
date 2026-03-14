import { Info } from 'lucide-react'
import { Field } from '@/components/atoms/field'
import { InputGroup, InputGroupAddon, InputGroupSwitch } from '@/components/atoms/input-group'
import { Label } from '@/components/atoms/label'
import { TooltipMacro } from '@/components/atoms/tooltip-macro'
import { cn } from '@/lib/utils/cn'

interface YoloModeProps {
  className?: string
  yolo: boolean
  onSetYolo: (yolo: boolean) => void

  canSelectModel?: boolean
  canSelectProvider?: boolean
  onSetReadOnly: (readOnly: boolean) => void
  onSetNetworkAccess: (networkAccess: boolean) => void
}

export const YoloMode = ({ className, yolo, onSetYolo: setYolo, onSetReadOnly, onSetNetworkAccess }: YoloModeProps) => {
  const handleYoloChange = (checked: boolean) => {
    setYolo(checked)
    if (checked) {
      onSetReadOnly(false)
      onSetNetworkAccess(true)
    }
  }

  return (
    <Field className={cn(className)}>
      <InputGroup className="justify-between">
        <InputGroupAddon className="w-44 justify-start">
          <Label className={cn(yolo && 'rainbow')} htmlFor="yolo">
            YOLO Mode
          </Label>
          <TooltipMacro tooltip="Skip all confirmations and run commands immediately">
            <Info className="h-4 w-4 text-primary cursor-help" />
          </TooltipMacro>
        </InputGroupAddon>
        <InputGroupSwitch checked={yolo} id="yolo" onCheckedChange={handleYoloChange} />
      </InputGroup>
    </Field>
  )
}
