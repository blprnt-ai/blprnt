import { Info } from 'lucide-react'
import { agentKindLabel } from '@/agent-kind-label'
import type { AgentKind } from '@/bindings'
import { Field } from '@/components/atoms/field'
import { InputGroup, InputGroupAddon } from '@/components/atoms/input-group'
import { Label } from '@/components/atoms/label'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/atoms/select'
import { TooltipMacro } from '@/components/atoms/tooltip-macro'
import { cn } from '@/lib/utils/cn'

interface AgentKindSelectProps {
  className?: string
  agentKind?: AgentKind
  onSetAgentKind: (agentKind: AgentKind) => void
}

export const AgentKindSelect = ({ className, agentKind = 'crew', onSetAgentKind = () => {} }: AgentKindSelectProps) => {
  return (
    <Field className={cn(className)}>
      <InputGroup className="justify-between w-full">
        <InputGroupAddon className="w-44 justify-start">
          <Label htmlFor="model">Mode</Label>
          <TooltipMacro tooltip={<ModeDescription />}>
            <Info className="h-4 w-4 text-primary cursor-help" />
          </TooltipMacro>
        </InputGroupAddon>
        <InputGroupAddon>
          <Select value={agentKind} onValueChange={(value) => onSetAgentKind(value as AgentKind)}>
            <SelectTrigger className="border-0" size="sm">
              <SelectValue placeholder="Select an agent kind">{agentKindLabel(agentKind)}</SelectValue>
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="crew">{agentKindLabel('crew')}</SelectItem>
              <SelectItem value="designer">{agentKindLabel('designer')}</SelectItem>

              {/* Temporarily disabled */}
              {/* <SelectItem value="execution">{agentKindLabel('execution')}</SelectItem> */}

              {/* Temporarily disabled */}
              {/* <SelectItem value="verification">{agentKindLabel('verification')}</SelectItem> */}
            </SelectContent>
          </Select>
        </InputGroupAddon>
      </InputGroup>
    </Field>
  )
}

const ModeDescription = () => {
  return (
    <div className="text-sm">
      <p>
        <span className="font-semibold">Crew:</span>
        <span className="font-medium">
          A blprnt architect that delegates tasks to blprnt planners, executors, and verifiers.
        </span>
        <br />
        <span className="italic font-regular">Useful for complex projects that require a lot of coordination.</span>
      </p>
    </div>
  )
}
