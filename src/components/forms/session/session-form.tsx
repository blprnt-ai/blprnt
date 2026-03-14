import { Info } from 'lucide-react'
import type { QueueMode } from '@/bindings'
import { Button } from '@/components/atoms/button'
import { Disclosure, DisclosureContent, DisclosureTrigger } from '@/components/atoms/disclosure'
import { Field } from '@/components/atoms/field'
import { InputGroup, InputGroupAddon } from '@/components/atoms/input-group'
import { Label } from '@/components/atoms/label'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/atoms/select'
import { TooltipMacro } from '@/components/atoms/tooltip-macro'
import type { SessionFormViewModel } from '@/components/forms/session/session-form.viewmodel'
import { ModelOverride } from './sections/model-override'
import { NetworkAccess } from './sections/network-access'
import { PersonalitySelect } from './sections/personality-select'
import { ReadOnly } from './sections/read-only'
import { SessionName } from './sections/session-name'
import { WebSearchToggle } from './sections/web-search-toggle'
import { YoloMode } from './sections/yolo-mode'

interface SessionConfigProps {
  session: SessionFormViewModel
  isNew?: boolean
  onSubmit: () => void
}

export const SessionForm = ({ session, isNew = false, onSubmit }: SessionConfigProps) => {
  const handleSubmit = () => onSubmit()

  return (
    <div className="flex h-full flex-col items-center justify-center">
      <div className="w-full max-w-6xl space-y-4">
        <SessionName sessionName={session.name} onSetSessionName={session.setName} />

        <ModelOverride modelOverride={session.modelOverride} onSetModelOverride={session.setModelOverride} />

        <WebSearchToggle
          webSearchEnabled={session.webSearchEnabled}
          onSetWebSearchEnabled={session.setWebSearchEnabled}
        />

        <PersonalitySelect personalityId={session.personalityId} onSetPersonalityId={session.setPersonalityId} />

        <QueueModeSelect queueMode={session.queueMode} onSetQueueMode={session.setQueueMode} />

        <Disclosure>
          <DisclosureTrigger>
            <div className="text-primary/60 mb-4 cursor-pointer hover:text-primary text-right">
              <Button variant="link">Advanced Options</Button>
            </div>
          </DisclosureTrigger>
          <DisclosureContent>
            <div className="w-full space-y-4">
              <YoloMode
                yolo={session.yolo}
                onSetNetworkAccess={session.setNetworkAccess}
                onSetReadOnly={session.setReadOnly}
                onSetYolo={session.setYolo}
              />

              <NetworkAccess
                isYolo={session.yolo}
                networkAccess={session.networkAccess}
                onSetNetworkAccess={session.setNetworkAccess}
              />

              <ReadOnly
                readOnly={session.readOnly}
                yolo={session.yolo}
                onSetReadOnly={session.setReadOnly}
                onSetYolo={session.setYolo}
              />
            </div>
          </DisclosureContent>
        </Disclosure>
      </div>

      <div className="flex justify-end w-full max-w-6xl">
        <Button
          className="h-12 w-full text-base"
          data-tour="session-create-submit"
          disabled={!session.isValid}
          size="lg"
          variant="outline"
          onClick={handleSubmit}
        >
          {isNew ? 'Create Session' : 'Update Session'}
        </Button>
      </div>
    </div>
  )
}

const QueueModeSelect = ({
  queueMode,
  onSetQueueMode,
}: {
  queueMode: string
  onSetQueueMode: (queueMode: QueueMode) => void
}) => {
  return (
    <Field>
      <InputGroup className="justify-between">
        <InputGroupAddon className="w-44 justify-start">
          <Label htmlFor="queue-mode">Prompt Processing</Label>
          <TooltipMacro
            tooltip={
              <div>
                <div>Determines how commands are processed.</div>
                <div className="text-xs text-muted-foreground mt-2">
                  <span className="font-bold">Queue:</span> Your prompts will be queued and executed in order, after the
                  previous prompt has been completed.
                </div>
                <div className="text-xs text-muted-foreground">
                  <span className="font-bold">Inject:</span> The agent will attempt to start processing your queued
                  prompts at the next available opportunity, usually in between tool calls.
                </div>
              </div>
            }
          >
            <Info className="h-4 w-4 text-primary cursor-help" />
          </TooltipMacro>
        </InputGroupAddon>
        <InputGroupAddon>
          <Select value={queueMode} onValueChange={onSetQueueMode}>
            <SelectTrigger className="border-0" size="xs">
              <SelectValue placeholder="Select a queue mode" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="queue">Queue</SelectItem>
              <SelectItem value="inject">Inject</SelectItem>
            </SelectContent>
          </Select>
        </InputGroupAddon>
      </InputGroup>
    </Field>
  )
}
