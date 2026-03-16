import { AlertTriangle, CheckCircle2, ChevronDownIcon, Circle, Loader2 } from 'lucide-react'
import { useEffect, useState } from 'react'
import { SubagentIcon } from '@/components/ai-elements/subagent'
import { Disclosure, DisclosureContent, DisclosureTrigger } from '@/components/atoms/disclosure'
import { useSessionPanelViewmodel } from '@/components/panels/session/session-panel.viewmodel'
import { SubAgentMessageModel } from '@/lib/models/messages/subagent-message.model'
import { cn } from '@/lib/utils/cn'
import { upperFirst } from '@/lib/utils/string'
import { SubagentConversation } from './subagent-conversation'

interface SubagentCardProps {
  messageKey: string
}

export const SubagentCard = ({ messageKey }: SubagentCardProps) => {
  const viewmodel = useSessionPanelViewmodel()
  const message = viewmodel.getMessageByKey(messageKey)
  const isSubagent = message instanceof SubAgentMessageModel
  const status = isSubagent ? message.status : undefined
  const [isExpanded, setIsExpanded] = useState(false)

  useEffect(() => {
    if (!isSubagent) return
    if (status !== 'in_progress') setIsExpanded(false)
  }, [isSubagent, status])

  if (!isSubagent) return null

  const agentName = message.input.name ?? 'Subagent'
  const isError = message.status === 'error'
  const statusIcon =
    message.status === 'completed'
      ? CheckCircle2
      : message.status === 'in_progress'
        ? Loader2
        : message.status === 'pending'
          ? Circle
          : AlertTriangle

  // Upper fist
  const agentKind = message.input.agent_kind
  const agentKindLabel = agentKind ? upperFirst(agentKind) : 'Subagent'
  const modelName = viewmodel.models.find((m) => m.slug === message.input.model_override)?.name

  return (
    <div className="flex flex-col gap-2">
      <Disclosure open={isExpanded} onOpenChange={setIsExpanded}>
        <DisclosureTrigger>
          <div
            className={cn(
              'border border-border bg-accent rounded-md p-2 pr-3 text-sm text-muted-foreground border-l-4',
              isError && 'border-destructive/50 bg-destructive/10',
            )}
          >
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-2">
                <SubagentIcon className={cn('size-5 text-primary/80', isError && 'text-destructive/70')} />
                <span>{agentKindLabel}:</span>
                <span className="font-medium">{agentName}</span>
              </div>
              <div className="flex items-center gap-2 text-xs text-muted-foreground">
                <span className="flex items-center gap-1">
                  {(() => {
                    const Icon = statusIcon
                    return (
                      <Icon
                        className={cn(
                          'size-3.5',
                          message.status === 'in_progress' && 'animate-spin',
                          message.status === 'completed' && 'text-success',
                          message.status === 'error' && 'text-destructive',
                        )}
                      />
                    )
                  })()}
                </span>
                <ChevronDownIcon className={cn('size-4 transition-transform', isExpanded && 'rotate-180')} />
              </div>
            </div>
          </div>
        </DisclosureTrigger>
        <DisclosureContent className="pl-4">
          <div className="mt-2 border-l border-border/40 pl-4">
            <div className="mb-2 flex flex-wrap items-center gap-3 text-[11px] text-muted-foreground">
              {modelName && (
                <div className="flex items-center gap-1">
                  <span className="text-foreground/40">Model:</span>
                  <span className="text-foreground/80">{modelName}</span>
                </div>
              )}
            </div>
            <SubagentConversation sessionId={message.subagentDetails?.sessionId} status={message.status} />
          </div>
        </DisclosureContent>
      </Disclosure>
    </div>
  )
}
