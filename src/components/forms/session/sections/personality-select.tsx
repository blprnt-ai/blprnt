import { Info } from 'lucide-react'
import { Field } from '@/components/atoms/field'
import { InputGroup, InputGroupAddon } from '@/components/atoms/input-group'
import { Label } from '@/components/atoms/label'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/atoms/select'
import { TooltipMacro } from '@/components/atoms/tooltip-macro'
import { useAppViewModel } from '@/hooks/use-app-viewmodel'
import { cn } from '@/lib/utils/cn'

interface PersonalitySelectProps {
  className?: string
  personalityId?: string
  onSetPersonalityId?: (personalityId: string) => void
}

const NO_PERSONALITY_VALUE = '__none__'

export const PersonalitySelect = ({
  className,
  personalityId = '',
  onSetPersonalityId = () => {},
}: PersonalitySelectProps) => {
  const app = useAppViewModel()

  return (
    <Field className={cn(className)}>
      <InputGroup className="justify-between">
        <InputGroupAddon className="w-44 justify-start">
          <Label htmlFor="personality">Personality</Label>
          <TooltipMacro
            tooltip={
              <div>
                <div>This affects the way the agent behaves in conversations.</div>
                <div className="text-xs text-muted-foreground mt-2">
                  This does not influence the code quality or performance. Only conversation style.
                </div>
              </div>
            }
          >
            <Info className="h-4 w-4 text-primary cursor-help" />
          </TooltipMacro>
        </InputGroupAddon>
        <InputGroupAddon>
          <Select
            value={personalityId || NO_PERSONALITY_VALUE}
            onValueChange={(value) => onSetPersonalityId(value === NO_PERSONALITY_VALUE ? '' : value)}
          >
            <SelectTrigger className="border-0" size="sm">
              <SelectValue placeholder="Select a personality">
                {app.personalities.find((p) => p.id === personalityId)?.name || 'Select a personality'}
              </SelectValue>
            </SelectTrigger>
            <SelectContent>
              <SelectItem value={NO_PERSONALITY_VALUE}>No personality</SelectItem>
              {app.personalities.map((p) => (
                <SelectItem key={p.id} value={p.id.toString()}>
                  {p.name}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </InputGroupAddon>
      </InputGroup>
    </Field>
  )
}
