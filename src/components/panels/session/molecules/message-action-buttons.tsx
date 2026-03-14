import { Maximize, Minimize, Rewind, Trash2 } from 'lucide-react'
import { Button } from '@/components/atoms/button'
import { CopyButton } from '@/components/atoms/copy-button'
import { TooltipMacro } from '@/components/atoms/tooltip-macro'
import { Confirm } from '@/components/dialogs/dope-dialog'
import { useSessionPanelViewmodel } from '@/components/panels/session/session-panel.viewmodel'
import type { PromptMessageModel } from '@/lib/models/messages/prompt-message.model'
import type { ResponseMessageModel } from '@/lib/models/messages/response-message.model'
import type { SignalMessageModel } from '@/lib/models/messages/signal-message.model'

interface MessageActionButtonsProps {
  message: PromptMessageModel | ResponseMessageModel | SignalMessageModel
  isExpanded: boolean
  onExpand: () => void
}

export const MessageActionButtons = ({ message, isExpanded, onExpand }: MessageActionButtonsProps) => {
  const viewmodel = useSessionPanelViewmodel()
  const canModify = !viewmodel.isRunning
  const handleCopy = () => {
    navigator.clipboard.writeText(message.content)
  }

  return (
    <div className="mt-auto flex items-center justify-end gap-1">
      <TooltipMacro withDelay tooltip="Copy message">
        <CopyButton
          className="size-7 text-muted-foreground/40 group-hover:text-muted-foreground hover:text-primary delay-0"
          content={message.content}
          variant="link"
          onClick={handleCopy}
        />
      </TooltipMacro>

      <TooltipMacro withDelay tooltip={isExpanded ? 'Collapse message' : 'Expand message'}>
        <Button
          className="size-7 text-muted-foreground/40 group-hover:text-muted-foreground hover:text-primary delay-35"
          size="icon"
          variant="link"
          onClick={onExpand}
        >
          {isExpanded ? <Minimize className="size-4" /> : <Maximize className="size-4" />}
        </Button>
      </TooltipMacro>

      <Confirm
        cancelLabel="Cancel"
        className="max-w-md"
        okLabel="Rewind"
        title="Rewind to Message"
        body={
          <span className="flex flex-col gap-1 text-sm text-muted-foreground text-center">
            <span>This will delete this message and all subsequent messages.</span>
            <span className="text-destructive">This action cannot be reverted.</span>
          </span>
        }
        onCancel={() => {}}
        onOk={() => viewmodel.rewindToMessage(message.id)}
      >
        <TooltipMacro withDelay tooltip="Rewind to this message">
          <Button
            className="size-7 text-muted-foreground/40 group-hover:text-muted-foreground hover:text-destructive delay-70"
            disabled={!canModify}
            size="icon"
            variant="link"
          >
            <Rewind className="size-4" />
          </Button>
        </TooltipMacro>
      </Confirm>

      <Confirm
        cancelLabel="Cancel"
        className="max-w-sm"
        okLabel="Delete"
        title="Delete Message"
        body={
          <span className="flex flex-col gap-1 text-sm text-muted-foreground text-center">
            <span>This will delete this message.</span>
            <span className="text-destructive">This action cannot be reverted.</span>
          </span>
        }
        onCancel={() => {}}
        onOk={() => viewmodel.deleteMessage(message.id)}
      >
        <TooltipMacro withDelay tooltip="Delete message">
          <Button
            className="size-7 text-muted-foreground/40 group-hover:text-muted-foreground hover:text-destructive delay-105"
            disabled={!canModify}
            size="icon"
            variant="link"
          >
            <Trash2 className="size-4" />
          </Button>
        </TooltipMacro>
      </Confirm>
    </div>
  )
}
